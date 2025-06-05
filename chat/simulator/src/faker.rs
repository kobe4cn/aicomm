use chrono::Utc;
use fake::{Dummy, Rng};

pub struct AppVersion;
pub struct SystemOs;
pub struct SystemArch;
pub struct SystemLanguage;
pub struct DateTime;
pub struct MessageType;

impl Dummy<AppVersion> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &AppVersion, rng: &mut R) -> Self {
        let x = rng.random_range(1..=9);
        let y = rng.random_range(0..=99);
        let z = rng.random_range(0..=99);
        format!("{}.{}.{}", x, y, z)
    }
}

impl Dummy<SystemOs> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &SystemOs, rng: &mut R) -> Self {
        let os = ["windows", "linux", "macos", "ios", "android"];
        let os = os[rng.random_range(0..os.len())];
        os.to_string()
    }
}

impl Dummy<SystemArch> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &SystemArch, rng: &mut R) -> Self {
        let arch = ["x86_64", "aarch64", "x86"];
        let arch = arch[rng.random_range(0..arch.len())];
        arch.to_string()
    }
}

impl Dummy<SystemLanguage> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &SystemLanguage, rng: &mut R) -> Self {
        let language = [
            "en", "zh", "es", "fr", "de", "it", "ja", "ko", "ru", "ar", "hi", "bn", "te", "ta",
            "ml", "th", "vi", "id", "ms", "fil", "ca", "eu", "gl", "ast", "ca",
        ];
        let language = language[rng.random_range(0..language.len())];
        language.to_string()
    }
}

impl Dummy<DateTime> for i64 {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &DateTime, _rng: &mut R) -> Self {
        //convert datetime to i64
        Utc::now().timestamp_millis()
    }
}

impl Dummy<MessageType> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &MessageType, rng: &mut R) -> Self {
        let message_type = ["text", "image", "audio", "video", "file"];
        let message_type = message_type[rng.random_range(0..message_type.len())];
        message_type.to_string()
    }
}
