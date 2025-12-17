/// File upload limits
pub mod file_limits {
    /// Maximum upload file size (50MB)
    pub const MAX_UPLOAD_SIZE: u64 = 50 * 1024 * 1024;
    
    /// Supported file extensions
    pub const SUPPORTED_EXTENSIONS: &[&str] = &["txt", "log", "profile"];
}

/// Performance thresholds for hotspot detection
pub mod thresholds {
    /// Time percentage threshold for critical hotspot
    pub const CRITICAL_TIME_PERCENTAGE: f64 = 50.0;
    
    /// Time percentage threshold for high severity hotspot
    pub const HIGH_TIME_PERCENTAGE: f64 = 30.0;
    
    /// Time percentage threshold for medium severity hotspot
    pub const MEDIUM_TIME_PERCENTAGE: f64 = 15.0;
    
    /// Time percentage threshold for low severity hotspot
    pub const LOW_TIME_PERCENTAGE: f64 = 5.0;
}

/// Performance score thresholds
pub mod scores {
    /// Excellent performance score threshold
    pub const EXCELLENT: u32 = 90;
    
    /// Good performance score threshold
    pub const GOOD: u32 = 70;
    
    /// Fair performance score threshold
    pub const FAIR: u32 = 50;
    
    /// Poor performance score threshold
    pub const POOR: u32 = 30;
}

