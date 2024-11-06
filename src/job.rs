#[derive(Clone)]
pub struct JobDimension {
    pub index: usize,
    pub span: usize,
}

impl JobDimension {
    pub fn new(span: usize) -> Self {
        if span == 0 {
            panic!("Span cannot be zero");
        }
        return Self {
            index: 0,
            span,
        };
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

pub type Job = Vec<JobDimension>;

pub fn create_job(dimensions: Vec<usize>) -> Job {
    let mut result = Job::with_capacity(dimensions.len());
    for dimension in dimensions {
        result.push(JobDimension::new(dimension));
    }
    return result;
}

pub trait JobIterator {
    fn next(&mut self) -> Option<Job>;
}

impl JobIterator for Job {
    fn next(&mut self) -> Option<Job> {
        if self.first()?.is_finished() {
            return None
        } 
        assert!(self.last()?.has_job());
        let result = self.clone();
        self.last_mut()?.next();
        loop {
            let mut repeat = false;
            for i in 1..self.len() {
                if self.get(i)?.is_finished() {
                    self.get_mut(i-1)?.next();
                    self.get_mut(i)?.reset();
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

pub trait Uri {
    fn to_uri(&self) -> String;
}

impl Uri for Job {
    fn to_uri(&self) -> String {
        let mut result = String::new();
        for dimension in self {
            result.push_str(&dimension.index.to_string());
            result.push('/');
        }
        result.pop();
        return result;
    }
}

pub trait Serial {
    fn to_string(&self) -> String;
}

impl Serial for Job {
    fn to_string(&self) -> String {
        let mut result = self.to_uri();
        result.push_str("\r\n");
        for dimension in self {
            result.push_str(&dimension.span.to_string());
            result.push('/');
        }
        result.pop();
        return result;
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_job_dimension_iterator() {
        let mut mirror = 0;
        let test = JobDimension::new(10);
        for job in test {
            assert_eq!(mirror, job);
            mirror += 1;
        }
        assert_eq!(mirror, 10);
    }

    #[test]
    fn test_job_dimension_bounds() {
        let mut test = JobDimension::new(2);
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
        let mut job = create_job(vec![2, 3, 4]);
        assert_eq!(job.len(), 3);
        assert_eq!(job.to_uri(), "0/0/0");
        job.next();
        assert_eq!(job.to_uri(), "0/0/1");
        job.next();
        assert_eq!(job.to_uri(), "0/0/2");
        job.next();
        assert_eq!(job.to_uri(), "0/0/3");
        job.next();
        assert_eq!(job.to_uri(), "0/1/0");
        job.next();
        assert_eq!(job.to_uri(), "0/1/1");
        job.next();
        assert_eq!(job.to_uri(), "0/1/2");
        job.next();
        assert_eq!(job.to_uri(), "0/1/3");
        job.next();
        assert_eq!(job.to_uri(), "0/2/0");
        job.next();
        assert_eq!(job.to_uri(), "0/2/1");
        job.next();
        assert_eq!(job.to_uri(), "0/2/2");
        job.next();
        assert_eq!(job.to_uri(), "0/2/3");
        job.next();
        assert_eq!(job.to_uri(), "1/0/0");
    }
}