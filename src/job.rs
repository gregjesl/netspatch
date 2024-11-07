use std::{collections::{HashMap, HashSet}, time::{SystemTime}, vec};

#[derive(Debug)]
pub enum Error {
    DimensionMismatch,
    ZeroSizedDimension,
    OutOfBounds,
    UnexpectedString,
    JobNotFound,
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct JobDimension {
    pub index: usize,
    pub span: usize,
}

impl JobDimension {
    pub fn new(span: usize) -> Result<Self, Error> {
        if span == 0 {
            return Err(Error::ZeroSizedDimension);
        }
        return Ok(Self {
            index: 0,
            span,
        });
    }

    pub fn has_job(&self) -> bool {
        return self.index < self.span;
    }

    pub fn is_finished(&self) -> bool {
        return !self.has_job();
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn bounds(&self) -> (f64, f64) {
        let lower = self.index as f64 / self.span as f64;
        let upper = if self.index + 1 == self.span {
            1.0
        } else {
            (self.index + 1) as f64 / self.span as f64
        };
        return (lower, upper);
    }

    pub fn as_fraction(&self) -> f64 {
        let (lower, upper) = self.bounds();
        return (lower + upper) / 2.0;
    }

    pub fn to_string(&self) -> String {
        return format!("{}/{}",self.index, self.span);
    }
}

impl std::iter::Iterator for JobDimension {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.has_job() {
            let result = self.index;
            self.index += 1;
            return Some(result);
        } else {
            return None;
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Job {
    index: Vec<JobDimension>,
}

impl Job {
    pub fn new(index: &Vec<usize>, dimensions: &Vec<usize>) -> Result<Self, Error> {
        if index.len() != dimensions.len() {
            return Err(Error::DimensionMismatch);
        }
        let order = dimensions.len();
        assert!(index.len() == order && dimensions.len() == order);

        let mut result = Vec::with_capacity(dimensions.len());
        for i in 0..order {
            if index.get(i).unwrap() >= dimensions.get(i).unwrap() {
                return Err(Error::OutOfBounds);
            }
            result.push(JobDimension {
                index: index.get(i).unwrap().clone(),
                span: dimensions.get(i).unwrap().clone(),
            });
        }
        return Ok(Self {
            index: result
        });
    }

    pub fn order(&self) -> usize {
        return self.index.len();
    }

    pub fn dimensions(&self) -> Vec<usize> {
        let mut result: Vec<usize> = Vec::with_capacity(self.order());
        for slice in &self.index {
            result.push(slice.span);
        }
        return result;
    }

    pub fn to_uri(&self) -> String {
        let mut result = String::new();
        for dimension in &self.index {
            result.push_str(&dimension.index.to_string());
            result.push('/');
        }
        result.pop();
        return result;
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for slice in &self.index {
            result.push_str(&format!("{}\r\n",slice.to_string()));
        }
        return result;
    }
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct JobStack {
    top: Job,
}

impl JobStack {
    pub fn new(dimensions: &Vec<usize>) -> Result<JobStack, Error> {
        return Ok(
            Self {
                top: Job::new(&vec![0; dimensions.len()], dimensions)?
            }
        );
    }

    pub fn order(&self) -> usize {
        return self.top.order();
    }

    pub fn is_empty(&self) -> bool {
        assert!(!self.top.index.is_empty());
        return self.top.index.first().unwrap().is_finished();
    }
}

impl std::iter::Iterator for JobStack {
    type Item = Job;

    fn next(&mut self) -> Option<Self::Item> {
        if self.top.index.first()?.is_finished() {
            return None
        } 
        assert!(self.top.index.last()?.has_job());
        let result = self.top.clone();
        self.top.index.last_mut()?.next();
        loop {
            let mut repeat = false;
            for i in 1..self.top.index.len() {
                if self.top.index.get(i)?.is_finished() {
                    self.top.index.get_mut(i-1)?.next();
                    self.top.index.get_mut(i)?.reset();
                    repeat = true;
                    break;
                }
            }
            if !repeat {
                break;
            }
        }
        return Some(result);
    }
}

pub struct JobManager {
    stack: JobStack,
    pending: HashMap<Job, SystemTime>,
    abandoned: HashSet<Job>,
}

impl JobManager {
    pub fn new(dimensions: &Vec<usize>) -> Result<Self, Error> {
        return Ok(Self {
            stack: JobStack::new(dimensions)?,
            pending: HashMap::new(),
            abandoned: HashSet::new(),
        });
    }

    fn set_pending(&mut self, job: &Job) {
        assert!(!self.pending.contains_key(job));
        self.pending.insert(job.clone(), SystemTime::now());
    }

    pub fn jobs_pending(&self) -> HashMap<Job, SystemTime> {
        return self.pending.clone();
    }

    pub fn jobs_abandonded(&self) -> HashSet<Job> {
        return self.abandoned.clone();
    }

    pub fn from_uri(&self, uri: String) -> Result<Job, Error> {
        let parts = uri.split('/');
        let mut index: Vec<usize> = Vec::with_capacity(self.stack.order());
        for part in parts {
            match part.parse::<usize>() {
                Ok(value) => index.push(value),
                Err(_) => return Err(Error::UnexpectedString),
            }
        }
        return Job::new(&index, &self.stack.top.dimensions());
    }

    pub fn pop(&mut self) -> Option<Job> {
        if !self.abandoned.is_empty() {
            let result = self.abandoned.iter().next().cloned().unwrap();
            self.abandoned.remove(&result);
            self.set_pending(&result);
            return Some(result);
        } else if !self.stack.is_empty() {
            let result = self.stack.next()?;
            assert!(!self.pending.contains_key(&result));
            self.set_pending(&result);
            assert!(self.pending.contains_key(&result));
            return Some(result);
        } else {
            return None;
        }
    }

    pub fn complete(&mut self, uri: String) -> Result<Job, Error> {
        let job = self.from_uri(uri)?;
        if self.pending.contains_key(&job) {
            self.pending.remove(&job);
            return Ok(job);
        } else if self.abandoned.contains(&job) {
            self.abandoned.remove(&job);
            return Ok(job);
        } else {
            return Err(Error::JobNotFound);
        }
    }

    pub fn abandon(&mut self, job: &Job) {
        assert!(self.pending.contains_key(&job));
        assert!(!self.abandoned.contains(&job));
        self.pending.remove(&job);
        self.abandoned.insert(job.clone());
    }

    pub fn is_finished(&self) -> bool {
        return self.stack.is_empty() && self.pending.is_empty() && self.abandoned.is_empty();
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_job_dimension_iterator() {
        let mut mirror = 0;
        let job = JobDimension::new(10).unwrap();
        for index in job {
            assert_eq!(mirror, index);
            mirror += 1;
        }
        assert_eq!(mirror, 10);
    }

    #[test]
    fn test_job_dimension_bounds() {
        let mut test = JobDimension::new(2).unwrap();
        {
            let (lower, upper) = test.bounds();
            assert_eq!(lower, 0.0);
            assert_eq!(upper, 0.5);
            assert_eq!(test.as_fraction(), 0.25);
        }
        test.next();
        {
            let (lower, upper) = test.bounds();
            assert_eq!(lower, 0.5);
            assert_eq!(upper, 1.0);
            assert_eq!(test.as_fraction(), 0.75);
        }
    }

    #[test]
    fn test_job_uri() {
        let mut stack = JobStack::new(&vec![2, 3, 4]).unwrap();
        assert_eq!(stack.order(), 3);
        for i in 0..2 {
            for j in 0..3 {
                for k in 0..4 {
                    assert!(!stack.is_empty());
                    let job = stack.next().unwrap();
                    assert_eq!(job.to_uri(), format!("{i}/{j}/{k}"));
                }
            }
        }
        assert!(stack.is_empty());
    }

    #[test]
    fn test_from_uri() {
        let dimensions = vec![2, 3, 4];
        let manager = JobManager::new(&dimensions).unwrap();
        let test = manager.from_uri("0/1/2".to_string()).unwrap();
        let actual = Job::new(&vec![0, 1, 2], &dimensions).unwrap();
        assert!(test.eq(&actual));
    }

    #[test]
    fn test_complete() {
        let dimensions = vec![2, 3, 4];
        let mut manager = JobManager::new(&dimensions).unwrap();
        assert_eq!(manager.jobs_pending().len(), 0);
        {
            let job = manager.pop().unwrap();
            assert_eq!(manager.jobs_pending().len(), 1);
            assert_eq!(manager.jobs_abandonded().len(), 0);
            let echo = manager.complete(job.to_uri()).unwrap();
            assert!(echo.eq(&job));
            assert_eq!(manager.jobs_pending().len(), 0);
            assert_eq!(manager.jobs_abandonded().len(), 0);
        }

        {
            let job = manager.pop().unwrap();
            assert_eq!(manager.jobs_pending().len(), 1);
            assert_eq!(manager.jobs_abandonded().len(), 0);
            manager.abandon(&job);
            assert_eq!(manager.jobs_pending().len(), 0);
            assert_eq!(manager.jobs_abandonded().len(), 1);
            let echo = manager.complete(job.to_uri()).unwrap();
            assert!(echo.eq(&job));
            assert_eq!(manager.jobs_pending().len(), 0);
            assert_eq!(manager.jobs_abandonded().len(), 0);
        }
    }
}