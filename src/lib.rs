pub mod browser;
pub mod cdp;
pub mod error;
pub mod step_frame;

//  Re-export commonly used items
pub use browser::chat::{ChatMessage, ChatUI, UserFeedback};
pub use browser::chrome::{ChromeDriver, ConnectionMode};
pub use cdp::{
    CdpCommand, CdpExecutor, CdpScript, CdpScriptGenerator, CdpValidator, CommandResult,
    CommandStatus, ErrorLocation, ExecutionReport, ValidationError, ValidationErrorType,
    ValidationResult,
};
pub use error::BrowserError;
pub use step_frame::{
    capture_step_frame, ActionInfo, CaptureOptions, DomInfo, InteractiveElement, ScreenshotFormat,
    ScreenshotInfo, StepFrame, TranscriptInfo,
};
