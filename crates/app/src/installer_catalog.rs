pub const TITLE: &str = "dlpad 개발용 설치";
pub const INTRO: &str = "서명되지 않은 개발용 설치입니다. 설치는 새 폴더에 복사합니다. 제거는 다시 검증한 뒤 명시적으로 확인하며 원본과 외부 단계 기록은 보존합니다.";
pub const LABELS: [&str; 4] = [
    "설치 명세 파일",
    "원본 파일 폴더",
    "설치 폴더",
    "외부 단계 기록 폴더",
];
pub const INSTALL: &str = "설치";
pub const CANCEL: &str = "검사 중단";
pub const CANCEL_INSTALL: &str = "설치 중단";
pub const CANCEL_REMOVE: &str = "제거 중단";
pub const CANCEL_LAUNCH: &str = "실행 취소";
pub const LOCATION_CANCELLED: &str =
    "검사를 중단했습니다. 다시 검증을 누르면 설치 위치와 기록을 다시 확인합니다.";
pub const VERIFY: &str = "다시 검증";
pub const IDLE: &str = "새 설치에는 네 경로를 입력하세요. 기존 설치 검증에는 설치 폴더와 외부 단계 기록 폴더만 입력한 뒤 다시 검증을 선택하세요.";
pub const INVALID: &str =
    "네 경로를 모두 로컬 절대 경로로 입력하세요. 기존 폴더에 덮어쓰지 않습니다.";
pub const INSTALLING: &str =
    "설치 파일과 단계 기록을 확인하고 복사 중입니다. 중단해도 만들어진 파일은 보존합니다.";
pub const VERIFYING: &str = "현재 설치 파일과 단계 기록을 읽어 확인 중입니다.";
pub const CANCELLING: &str = "중단을 요청했습니다. 작업이 종료될 때까지 기다립니다.";
pub const CLOSE_WAIT: &str = "중단을 요청했습니다. 작업 종료를 확인한 뒤 창을 닫습니다.";
pub const INSTALLED: &str = "설치 파일과 단계 기록을 확인했습니다. 다시 검증할 수 있습니다.";
pub const VERIFIED: &str = "기존 앱 확인을 마쳤습니다. 앱 실행을 누르세요.";
pub const FAILED: &str =
    "확인하지 못했습니다. 경로와 원본 파일을 확인하세요. 만들어진 파일은 보존합니다.";
pub const CANCELLED: &str = "작업을 중단했습니다. 만들어진 파일과 단계 기록은 보존합니다.";
pub const TIMED_OUT: &str = "작업 시간 제한을 넘었습니다. 만들어진 파일과 단계 기록은 보존합니다.";
pub const WORKER_FAILED: &str =
    "작업 종료를 확인했습니다. 결과를 받지 못해 설치 성공으로 표시하지 않습니다.";

pub const EXISTING: &str = "설치 폴더 또는 단계 기록 폴더가 이미 있습니다. 새 경로를 입력하세요. 기존 자료는 보존했습니다.";
pub const MISSING: &str = "파일 또는 상위 폴더를 찾지 못했습니다. 입력한 경로를 확인하세요.";
pub const SOURCE_CHANGED: &str = "원본 파일이나 기록을 확인하지 못했습니다. 명세와 원본이 같은 묶음인지 확인하세요. 만들어진 파일은 보존합니다.";
pub const NO_ACCESS: &str =
    "파일에 접근하지 못했습니다. 경로의 읽기·쓰기 권한과 사용 중인 프로그램을 확인하세요.";

pub const REMOVE: &str = "제거 요청";
pub const CONFIRM_REMOVE: &str = "제거 확인";
pub const BACK: &str = "돌아가기";
pub const REMOVE_CONFIRM: &str = "현재 입력한 설치 폴더의 고정 파일만 제거합니다. 원본과 단계 기록은 보존합니다. 제거 확인을 선택하면 되돌릴 수 없는 제거를 시작합니다.";
pub const REMOVING: &str =
    "현재 설치를 다시 확인하고 고정 파일을 제거 중입니다. 원본과 단계 기록은 보존합니다.";
pub const REMOVE_CANCELLING: &str =
    "제거 중단을 요청했습니다. 작업 종료를 기다립니다. 이미 제거된 파일은 복구하지 않습니다.";
pub const REMOVE_CLOSE_WAIT: &str = "제거 중단을 요청했습니다. 작업 종료를 확인한 뒤 창을 닫습니다. 이미 제거된 파일은 복구하지 않습니다.";
pub const REMOVED: &str =
    "설치 파일을 제거하고 설치 폴더의 부재를 확인했습니다. 원본과 단계 기록은 보존했습니다.";
pub const REMOVAL_PARTIAL: &str = "제거가 끝나지 않았습니다. 일부 파일은 이미 제거됐을 수 있습니다. 남은 파일과 단계 기록은 보존합니다.";
pub const REMOVAL_REFUSED: &str =
    "제거를 시작하지 못했습니다. 현재 설치와 단계 기록을 다시 확인하세요. 자료를 보존합니다.";

pub const REMOVAL_WORKER_FAILED: &str = "제거 작업 종료를 확인했지만 결과를 받지 못했습니다. 제거 여부를 확인할 수 없습니다. 현재 파일과 단계 기록을 다시 검증하세요.";

pub const INVALID_ROOTS: &str = "설치 폴더와 외부 단계 기록 폴더를 로컬 절대 경로로 입력하세요.";

pub const PICK_MANIFEST: &str = "명세 파일 선택";
pub const PICKER_TITLE: &str = "설치 명세 파일 선택";
pub const PICKER_FILTER: &str = "JSON 명세 파일";
pub const PICKER_FAILED: &str = "명세 파일을 선택하지 못했습니다. 기존 입력은 유지합니다.";
pub const PICKER_CHANGED: &str = "선택 중 입력 상태가 바뀌어 결과를 적용하지 않았습니다.";
pub const PICKER_CLOSE_FAILED: &str = "선택창을 닫지 못했습니다. 선택창에서 취소를 누르세요.";

// Optional tool order is yt-dlp, FFmpeg, Deno; unchecked by default.
pub const OPTIONAL_TOOLS: [&str; 3] = ["yt-dlp 함께 설치", "FFmpeg 함께 설치", "Deno 함께 설치"];
pub const TOOLS_HINT: &str =
    "선택한 도구를 앱 데이터 폴더에 설치합니다. 실패해도 앱 설치는 보존됩니다.
설치 후 앱의 도구 화면에서 선택·사용하세요.";
pub const TOOLS_DATA_UNAVAILABLE: &str =
    "앱 데이터 폴더를 찾지 못했습니다. 도구를 선택하지 않으면 앱만 설치할 수 있습니다.";

pub const TOOL_PATHS_REJECTED: &str = "도구 저장 위치가 원본·앱 설치·단계 기록 위치와 겹칩니다. 서로 다른 경로를 사용하세요. 설치를 시작하지 않았습니다.";

pub const PACKAGED_TITLE: &str = "dlpad 개발용 설치";
pub const PACKAGED_INTRO: &str = "포함된 앱을 새 폴더에 설치합니다. 서명되지 않은 개발 후보입니다. 설치 위치를 입력하거나 상위 폴더를 선택하세요.";
pub const PACKAGED_IDLE: &str = "설치 위치를 지정한 뒤 설치를 누르세요. 설치 기록 위치는 자동으로 정하며 기존 폴더는 덮어쓰지 않습니다.";

pub const TARGET_PICKER: &str = "상위 폴더 선택";
pub const TARGET_PICKER_TITLE: &str = "dlpad를 설치할 상위 폴더 선택";
pub const PACKAGED_JOURNAL_LABEL: &str = "설치 기록 위치 (자동 결정)";
pub const PACKAGED_INVALID: &str = "기존 상위 폴더 안의 새 설치 폴더를 입력하거나 상위 폴더를 선택하세요. 기존 설치에 덮어쓰지 않습니다.";

// Session-only appearance; native ID210 retains ComboBox semantics.
pub const PREPARATION_HEADING: &str = "설치 준비";
pub const OPTIONAL_TOOLS_HEADING: &str = "다운로드 도구 (선택)";
pub const THEME_LABEL: &str = crate::catalog::THEME;
pub const THEME_CHOICES: [&str; 3] = [
    crate::catalog::SYSTEM,
    crate::catalog::LIGHT,
    crate::catalog::DARK,
];

pub const SCALE_LABEL: &str = crate::catalog::SCALE;
pub const SCALE_CHOICES: [&str; 3] = ["100%", "150%", "200%"];
pub const APPEARANCE_HELP: &str =
    "시스템을 선택하면 Windows 색상을 따릅니다. 모양과 배율은 현재 설치 창에만 적용됩니다.";

// Explicit installed-app action; launching never changes verification/cleanup authority.
pub const RUN_APP: &str = "앱 실행";
pub const RUN_STARTING: &str = "설치된 앱을 확인하고 실행합니다.";
pub const RUNNING: &str = "앱 실행을 요청했습니다.";
pub const RUN_FAILED: &str = "앱을 실행하지 못했습니다. 다시 검증한 뒤 실행하세요.";
pub const RUN_CLEANUP_UNKNOWN: &str = "앱 실행 프로세스의 종료를 확인하지 못했습니다.";

pub const RUN_FAILED_AFTER_START: &str =
    "앱 실행 후 오류가 발생했습니다. 다시 검증한 뒤 실행하세요.";

/// An exit code has at most ten decimal digits; no tool output/path is included.
pub fn run_exit_failure(code: u32) -> String {
    format!("앱이 오류로 종료되었습니다 (종료 코드 {code}). 다시 검증한 뒤 실행하세요.")
}

// Presentation only: verified is the existing current-consistency result, never
// an authority to launch without the retained worker's fresh typed inspection.
pub const COMPLETED_HEADING: &str = "실행 준비 완료";
pub const COMPLETED_INTRO: &str =
    "앱 실행을 눌러 설치된 앱을 시작하세요. 앱이 시작되면 설치 창은 자동으로 닫힙니다.";
pub fn stage_copy(verified: bool, packaged: bool, failed: bool) -> (&'static str, &'static str) {
    if verified && !failed {
        (COMPLETED_HEADING, COMPLETED_INTRO)
    } else {
        (
            PREPARATION_HEADING,
            if packaged { PACKAGED_INTRO } else { INTRO },
        )
    }
}
/// Launch admission failure can retain verified=true for deliberate retry.
/// This changes only heading/help copy, never the action authority or buttons.
pub fn failure_stage_message(message: &str) -> bool {
    matches!(
        message,
        RUN_FAILED | RUN_FAILED_AFTER_START | RUN_CLEANUP_UNKNOWN | WORKER_FAILED
    )
}

// Read-only location admission. Presence never authorizes ownership or upgrades.
pub const CHECKING_LOCATION: &str =
    "설치 위치와 기존 앱 파일을 확인 중입니다. 입력은 계속 수정할 수 있습니다.";
pub const EXISTING_LOCATION: &str = "설치 폴더나 설치 기록이 이미 있습니다. 다시 검증 후 앱을 실행하거나 새 폴더를 선택하세요. 기존 설치 덮어쓰기·다른 버전으로 업그레이드는 지원하지 않습니다.";
pub const LOCATION_UNAVAILABLE: &str = "설치 위치를 확인하지 못했습니다. 기존 상위 폴더 안의 새 폴더를 선택하거나 기존 설치를 다시 검증하세요. 기존 자료는 보존합니다.";
pub const INPUT_CHANGED: &str =
    "확인 중 경로가 바뀌어 이전 결과를 적용하지 않았습니다. 현재 경로를 다시 확인하세요.";

pub const EXISTING_HEADING: &str = "기존 설치 확인";
pub const EXISTING_INTRO: &str = "현재 설치 위치와 기록을 다시 검증하세요. 검증 후 앱을 실행할 수 있습니다. 새 설치는 다른 빈 폴더를 선택하세요.";

// Essential readiness feedback is visible, never tooltip-only.
pub const CHECKING_HEADING: &str = "설치 확인 중";
pub const CHECKING_INTRO: &str = "기존 설치가 있으면 앱 파일과 설치 기록을 읽어 확인합니다. 확인하는 동안 앱 실행은 잠시 사용할 수 없습니다.";
pub const VERIFYING_HEADING: &str = "기존 앱 확인 중";
pub const VERIFYING_INTRO: &str =
    "앱 파일과 설치 기록이 일치하는지 확인합니다. 완료되면 아래 앱 실행을 누르세요.";
pub const INSTALLING_HEADING: &str = "앱 설치 중";
pub const REMOVING_HEADING: &str = "앱 제거 중";
