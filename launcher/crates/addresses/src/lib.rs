//! The outside addresses the launcher has built in, vanilla's aside, in one place.

/// The players' page on connecting an AI agent (the website's docs/agents.md).
pub const AGENT_DOCS: &str = "https://playsunrise.net/docs/agents";

/// The mod registry's policy, which the launcher's mod pages link.
pub const MOD_POLICY: &str = "https://playsunrise.net/mods/policy";

/// The telemetry ingest server (`crates/sunrise-telemetry-server` in the game repository): opt-in
/// telemetry, the "I'm playing" ping, benchmark uploads (`SUNRISE_TELEMETRY_URL` overrides it).
pub const TELEMETRY: &str = "https://telemetry.playsunrise.net";

/// The GitHub profile of who made the launcher's look, credited in Settings' About (its picture is
/// `<this>.png`).
pub const LAUNCHER_LOOK_CREDIT: &str = "https://github.com/Xmilon";

/// The official launcher installer's product home.
pub const LAUNCHER_ABOUT: &str = "https://playsunrise.net";

/// The separately branded arismoko fork launcher installer's product home.
pub const FORK_LAUNCHER_ABOUT: &str = "https://github.com/sunrise-org/sunrise";

/// Cargo's canonical source identity for the crates.io registry.
pub const CARGO_CRATES_IO_SOURCE: &str = "registry+https://github.com/rust-lang/crates.io-index";

/// The pinned sunrise-mixins archive the launcher downloads for `[[native.mixin]]` builds.
pub const SUNRISE_MIXINS_SOURCE: &str =
    "https://codeload.github.com/horizon-rust/horizon-mixins/tar.gz";

/// Where the game repository releases the prebuilt sunrise-mixins tool; a release's tag and file follow.
pub const MIXINS_TOOL_RELEASES: &str = "https://github.com/sunrise-org/sunrise/releases/download";

/// Where the game repository publishes each game build's mod SDK (`sdk-<build id>`) and the game
/// releases whose `sdk.json` names their build; a release's tag and file follow.
/// `SUNRISE_SDK_BASE` overrides it.
pub const SDK_RELEASES: &str = "https://github.com/sunrise-org/sunrise/releases/download";
