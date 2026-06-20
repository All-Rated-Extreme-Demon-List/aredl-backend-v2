use actix_web::error::BlockingError;
use actix_web::http::StatusCode;
use actix_web::{Error as ActixError, HttpResponse, ResponseError};
use diesel::result::DatabaseErrorKind;
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Display, Formatter};
use url::ParseError;

#[derive(Debug)]
pub enum ConfigError {
    MissingSecret {
        name: String,
    },
    SecretFileRead {
        name: String,
        path: String,
        source: std::io::Error,
    },
    InvalidValue {
        name: String,
        message: String,
    },
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingSecret { name } => {
                write!(f, "Missing required config secret: {name}")
            }
            ConfigError::SecretFileRead { name, path, source } => {
                write!(
                    f,
                    "Failed to read config secret {name} from {path}: {source}"
                )
            }
            ConfigError::InvalidValue { name, message } => {
                write!(f, "Invalid config value {name}: {message}")
            }
        }
    }
}

impl StdError for ConfigError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ConfigError::SecretFileRead { source, .. } => Some(source),
            ConfigError::MissingSecret { .. } | ConfigError::InvalidValue { .. } => None,
        }
    }
}

#[derive(Debug)]
pub enum StartupError {
    Config(ConfigError),
    Init(String),
    Io(std::io::Error),
}

impl Display for StartupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StartupError::Config(error) => Display::fmt(error, f),
            StartupError::Init(message) => f.write_str(message),
            StartupError::Io(error) => Display::fmt(error, f),
        }
    }
}

impl StdError for StartupError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            StartupError::Config(error) => Some(error),
            StartupError::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for StartupError {
    fn from(error: std::io::Error) -> Self {
        StartupError::Io(error)
    }
}

impl From<ConfigError> for StartupError {
    fn from(error: ConfigError) -> Self {
        StartupError::Config(error)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error_status_code: u16,
    pub error_message: String,
}

impl ApiError {
    pub fn new(error_status_code: u16, error_message: &str) -> ApiError {
        ApiError {
            error_status_code,
            error_message: error_message.to_string(),
        }
    }
}

impl From<ApiError> for StartupError {
    fn from(error: ApiError) -> Self {
        StartupError::Init(error.to_string())
    }
}

macro_rules! api_error_status_constructors {
    ($($name:ident => $status:ident),+ $(,)?) => {
        #[expect(non_snake_case, reason = "HTTP status codes conventions")]
        impl ApiError {
            $(
                pub fn $name(error_message: impl ToString) -> ApiError {
                    ApiError {
                        error_status_code: StatusCode::$status.as_u16(),
                        error_message: error_message.to_string(),
                    }
                }
            )+
        }
    };
}

api_error_status_constructors! {
    BadRequest => BAD_REQUEST,
    Unauthorized => UNAUTHORIZED,
    PaymentRequired => PAYMENT_REQUIRED,
    Forbidden => FORBIDDEN,
    NotFound => NOT_FOUND,
    MethodNotAllowed => METHOD_NOT_ALLOWED,
    NotAcceptable => NOT_ACCEPTABLE,
    ProxyAuthenticationRequired => PROXY_AUTHENTICATION_REQUIRED,
    RequestTimeout => REQUEST_TIMEOUT,
    Conflict => CONFLICT,
    Gone => GONE,
    LengthRequired => LENGTH_REQUIRED,
    PreconditionFailed => PRECONDITION_FAILED,
    PayloadTooLarge => PAYLOAD_TOO_LARGE,
    UriTooLong => URI_TOO_LONG,
    UnsupportedMediaType => UNSUPPORTED_MEDIA_TYPE,
    RangeNotSatisfiable => RANGE_NOT_SATISFIABLE,
    ExpectationFailed => EXPECTATION_FAILED,
    ImATeapot => IM_A_TEAPOT,
    MisdirectedRequest => MISDIRECTED_REQUEST,
    UnprocessableEntity => UNPROCESSABLE_ENTITY,
    Locked => LOCKED,
    FailedDependency => FAILED_DEPENDENCY,
    UpgradeRequired => UPGRADE_REQUIRED,
    PreconditionRequired => PRECONDITION_REQUIRED,
    TooManyRequests => TOO_MANY_REQUESTS,
    RequestHeaderFieldsTooLarge => REQUEST_HEADER_FIELDS_TOO_LARGE,
    UnavailableForLegalReasons => UNAVAILABLE_FOR_LEGAL_REASONS,
    InternalServerError => INTERNAL_SERVER_ERROR,
    NotImplemented => NOT_IMPLEMENTED,
    BadGateway => BAD_GATEWAY,
    ServiceUnavailable => SERVICE_UNAVAILABLE,
    GatewayTimeout => GATEWAY_TIMEOUT,
    HttpVersionNotSupported => HTTP_VERSION_NOT_SUPPORTED,
    VariantAlsoNegotiates => VARIANT_ALSO_NEGOTIATES,
    InsufficientStorage => INSUFFICIENT_STORAGE,
    LoopDetected => LOOP_DETECTED,
    NotExtended => NOT_EXTENDED,
    NetworkAuthenticationRequired => NETWORK_AUTHENTICATION_REQUIRED,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.error_message.as_str())
    }
}

impl From<DieselError> for ApiError {
    fn from(error: DieselError) -> Self {
        match error {
            DieselError::DatabaseError(kind, err) => match kind {
                DatabaseErrorKind::UniqueViolation
                | DatabaseErrorKind::SerializationFailure
                | DatabaseErrorKind::RestrictViolation
                | DatabaseErrorKind::ExclusionViolation => ApiError::Conflict(err.message()),
                DatabaseErrorKind::ForeignKeyViolation
                | DatabaseErrorKind::NotNullViolation
                | DatabaseErrorKind::CheckViolation => ApiError::UnprocessableEntity(err.message()),
                DatabaseErrorKind::ClosedConnection => {
                    ApiError::ServiceUnavailable("Database unavailable, please try again later")
                }
                DatabaseErrorKind::UnableToSendCommand
                | DatabaseErrorKind::ReadOnlyTransaction
                | _ => ApiError::InternalServerError(format!(
                    "Internal Database error: {}",
                    err.message()
                )),
            },
            DieselError::NotFound => ApiError::NotFound("Not found"),
            err @ DieselError::InvalidCString(_)
            | err @ DieselError::QueryBuilderError(_)
            | err @ DieselError::DeserializationError(_)
            | err @ DieselError::SerializationError(_)
            | err @ DieselError::RollbackErrorOnCommit { .. }
            | err @ DieselError::RollbackTransaction
            | err @ DieselError::AlreadyInTransaction
            | err @ DieselError::NotInTransaction
            | err @ DieselError::BrokenTransactionManager
            | err => ApiError::InternalServerError(format!("Unexpected Internal error: {}", err)),
        }
    }
}

impl From<BlockingError> for ApiError {
    fn from(_error: BlockingError) -> Self {
        ApiError::InternalServerError("Internal server error")
    }
}

impl From<ParseError> for ApiError {
    fn from(error: ParseError) -> Self {
        ApiError::InternalServerError(format!("Failed to parse URL: {}", error))
    }
}

impl From<ConfigError> for ApiError {
    fn from(error: ConfigError) -> Self {
        ApiError::InternalServerError(format!("Configuration error: {}", error))
    }
}

impl From<ActixError> for ApiError {
    fn from(error: ActixError) -> Self {
        let response_error = error.as_response_error();
        ApiError {
            error_status_code: response_error.status_code().as_u16(),
            error_message: error.to_string(),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.error_status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();

        let error_message = match status_code.as_u16() < 500 {
            true => self.error_message.clone(),
            false => "Internal server error".to_string(),
        };

        HttpResponse::build(status_code).json(json!({"message": error_message}))
    }
}
