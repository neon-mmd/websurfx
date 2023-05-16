#[derive(Debug)]
pub enum ReqwestError{
  NotFound,
  Timeout,
  Forbidden,
  AccessDenied,
  TooManyRequests
}
