use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Backend não disponível")]
    Unavailable,

    #[error("Operação não suportada neste MVP: {0}")]
    Unsupported(String),

    #[error("Pacote não encontrado: {0}")]
    NotFound(String),

    #[error("Permissão negada: {0}")]
    PermissionDenied(String),

    #[error("Identificador de pacote inválido: {0}")]
    InvalidPackageId(String),

    #[error("Falha ao executar comando: {0}")]
    CommandFailed(String),

    #[error("Erro de E/S: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type BackendResult<T> = Result<T, BackendError>;
