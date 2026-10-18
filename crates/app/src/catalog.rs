//! Central Korean UI catalog; no secrets or execution details in display state.
pub const TITLE: &str = "dlpad · 네이티브 검증";
pub const URL: &str = "영상 URL";
pub const PLACEHOLDER: &str = "https://…";
pub const LIBRARY: &str = "파일 목록";
pub const DOWNLOADS: &str = "다운로드";
pub const SETTINGS: &str = "설정";
pub const SYSTEM: &str = "시스템";
pub const LIGHT: &str = "밝게";
pub const DARK: &str = "어둡게";
pub const THEME: &str = "화면 테마";
pub const EMPTY: &str = "영상 URL을 입력하세요. 이 화면은 렌더러와 입력 검증용입니다.";
pub const FIXTURE: &str = "합성 파일 10,000개";
pub const LOCAL: &str = "로컬 파일";
pub const METRICS: &str = "수신·송신 측정 불가 · 미디어 처리 측정 불가 · 디스크 측정 불가";
pub const IME: &str = "한글 입력 검증";
pub const IME_HELP: &str = "조합 입력, 붙여넣기, 방향키, 선택과 삭제를 확인하세요.";
pub const SCALE: &str = "화면 배율";
pub const STATUS: &str = "S00 개발 화면 · 다운로드 기능 준비 중";

pub const DIAGNOSTICS: &str = "실패 원인 보기";
pub const COPY_REPORT: &str = "검토한 자료 복사";
pub const SAVE_REPORT: &str = "새 파일로 저장";
pub const EXPORT_PATH: &str = "진단 자료 저장 경로";
pub const REPORT_REVIEW: &str =
    "자료는 외부로 자동 전송되지 않습니다. 복사하거나 저장하기 전에 내용을 확인하세요.";
pub const REMOVE_LOGS: &str = "로그 제외";
pub const SAVE_ROOT: &str = "저장 폴더";
pub const REFRESH: &str = "목록 새로고침";
pub const NEXT: &str = "다음 페이지";

pub const TOOL_PATH: &str = "기존 yt-dlp 실행 파일";
pub const START: &str = "다운로드 시작";
pub const CANCEL: &str = "작업 취소";
pub const DOWNLOADING: &str = "다운로드 준비·실행 중";
pub const DOWNLOADED: &str = "최종 파일 확인 완료";
pub const FAILED: &str = "작업이 실패했습니다. 실패 원인 보기를 확인하세요.";
pub const CLOSE_ACTIVE: &str = "작업 중입니다. 중단 후 종료하거나 돌아가세요.";
pub const CLOSE_WAIT_TITLE: &str = "종료 준비 중";
pub const CLOSE_WAIT_HELP: &str = "작업 정리를 기다리고 있습니다. 완료되면 앱이 종료됩니다.";
pub const STOP_EXIT: &str = "중단 후 종료";
pub const RETURN: &str = "돌아가기";
pub const QUALITY: &str = "품질 높이 (0: 최고 품질)";
pub const RANGE_START: &str = "시작 (시작과 끝을 모두 비우면 전체 영상)";
pub const RANGE_END: &str = "끝, HH:MM:SS.mmm";
pub const CUT_COST: &str =
    "빠른 구간 저장은 키프레임 차이가 생길 수 있습니다. 정확한 자르기는 재인코딩 비용이 있습니다.";
pub const CANCELLED: &str =
    "작업을 취소했습니다. 미완료 파일은 보존하지만 이력에서 재개하지 않습니다.";

pub const MARKDOWN: &str = "Markdown";
pub const JSON: &str = "JSON";
pub const NETWORK_PROCESS: &str = "수신·송신 측정 불가 · 미디어 처리 측정 불가";
pub const FREE_SPACE: &str = "저장 폴더 여유";
pub const FREE_SPACE_HELP: &str = "아래 저장 폴더가 위치한 볼륨에서 현재 사용자에게 사용 가능한 공간입니다. 표시한 드라이브 문자는 저장 경로이며, 다른 볼륨에 연결된 폴더에서는 그 볼륨의 공간을 측정합니다.";
pub const UNKNOWN: &str = "측정 불가";

pub const WATCH_FAILED: &str = "파일 변경 알림을 받지 못했습니다. 목록 새로고침을 사용하세요.";
pub const SEPARATE_DATA: &str = "저장 폴더와 앱 데이터 폴더는 서로 별도로 선택하세요.";

pub const LOG_INDEX: &str = "로그 번호 (0부터)";
pub const EDIT_LOG: &str = "선택한 로그 수정";
pub const APPLY_EDIT: &str = "정제 후 반영";
pub const OMIT_LOG: &str = "선택한 로그 제외";
pub const MPD_PATH: &str = "추가할 로컬 MPD 경로 (최대 1 MiB)";
pub const ATTACH_MPD: &str = "구조만 첨부";
pub const MPD_BASE_URL_NODE: &str = "기준 주소";
pub const MPD_SEGMENT_LIST_NODE: &str = "구간 목록";
pub const MPD_INITIALIZATION_NODE: &str = "초기화 데이터";
pub const MPD_SEGMENT_URL_NODE: &str = "미디어 구간";
pub const MPD_BYTE_RANGE: &str = "바이트 범위";
pub const MPD_MEDIA_BYTE_RANGE: &str = "미디어 바이트 범위";
pub const INVALID_LOG_INDEX: &str = "현재 보고서에 있는 로그 번호를 입력하세요.";

pub const PLAN: &str = "실행 계획 미리보기 (옵션 버전은 검증되지 않음)";
pub const OMITTED_SOURCE: &str = "[원본 URL 생략]";
pub const NO_REPORT: &str =
    "작업이 실패하면 이곳에서 근거를 확인하고 전달 자료를 만들 수 있습니다.";

pub const DOWNLOAD_SPEED: &str = "다운로드";
pub const ETA: &str = "남은 시간";
pub const SECONDS: &str = "초";

pub const INTERFACE: &str = "인터페이스";
pub const INTERFACE_HELP: &str = "선택한 인터페이스의 전체 트래픽이며 다른 앱도 포함합니다. 다운로드 속도와는 별도 관측입니다. 빈칸을 적용하면 끕니다.";
#[cfg(target_os = "linux")]
pub const INTERFACE_EXAMPLE: &str = "예: eth0 또는 lo";
#[cfg(target_os = "windows")]
pub const INTERFACE_EXAMPLE: &str = "예: 이더넷 또는 Wi-Fi (Windows의 정확한 이름)";
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub const INTERFACE_EXAMPLE: &str = "이 OS에서는 인터페이스 관측을 지원하지 않습니다";
pub const NETWORK_COUNTER_UNAVAILABLE: &str = "인터페이스 수신·송신 측정 미지원";
pub const NETWORK_COUNTER_UNAVAILABLE_DETAILS: &str = "다운로드 속도·저장 설정 안내";
pub const NETWORK_COUNTER_UNAVAILABLE_HELP: &str = "이 OS에서는 인터페이스 수신·송신을 측정할 수 없습니다. 다운로드 속도는 작업 진행 정보에서 별도로 표시합니다. 저장된 인터페이스 설정은 유지합니다.";
pub const APPLY_INTERFACE: &str = "관측 적용";
#[cfg(target_os = "windows")]
pub const INVALID_INTERFACE: &str = "연결 이름의 길이와 문자를 확인하세요.";
#[cfg(not(target_os = "windows"))]
pub const INVALID_INTERFACE: &str = "64자 이하의 영문·숫자·점·밑줄·하이픈 이름을 입력하세요.";
pub const RECEIVED: &str = "수신";
pub const SENT: &str = "송신";
pub const NETWORK_UNKNOWN: &str = "수신·송신 측정 불가";
pub const NETWORK_PREPARING: &str = "수신·송신 관측 준비 중";
pub const MEDIA_UNKNOWN: &str = "미디어 처리 측정 불가";

pub const APPLY_ROOT: &str = "저장 위치 적용";
pub const DEFAULT_ROOT: &str = "기본 문서 폴더 사용";
pub const DATA_REQUIRED: &str =
    "앱 데이터 위치를 찾지 못했습니다. 폴더 권한과 사용자 프로필을 확인하세요.";
pub const INVALID_ROOT: &str =
    "저장 위치를 확인하세요. 절대 경로와 폴더 권한을 확인하고 앱 데이터와 별도로 선택하세요.";

pub const DOCUMENTS_UNAVAILABLE: &str =
    "문서 폴더 위치를 찾지 못했습니다. 저장할 폴더의 절대 경로를 직접 입력하고 적용하세요.";
pub const LOCATION_FAILED: &str =
    "저장 위치를 준비하지 못했습니다. 폴더 권한을 확인하거나 다른 폴더를 선택하세요.";

pub const FIXTURE_DOCUMENTS: &str =
    "합성 시험에서는 시험용 문서 폴더를 지정한 실행에서만 기본 저장 위치를 준비합니다.";

pub const CUT_MODE: &str = "구간 자르기 방식";
pub const FAST_CUT: &str = "빠르게 (키프레임 기준)";
pub const ACCURATE_CUT: &str = "정확하게 (재인코딩)";
pub const ADVANCED_OPEN: &str = "고급 옵션 펼치기";
pub const ADVANCED_CLOSE: &str = "고급 옵션 접기";
pub const ADVANCED_HELP: &str =
    "빈칸은 기본값을 사용합니다. 입력한 설정은 실행 계획에서 확인하세요.";
pub const HELP_DEFAULT: &str = "기본값";
pub const HELP_EXAMPLE: &str = "예";
pub const HELP_CONFLICT: &str = "충돌";
pub const HELP_VERSION: &str = "설치 버전";
pub const QUALITY_NUMBER: &str = "품질 높이는 0 또는 144~4320 정수로 입력하세요.";

pub const FFMPEG_PATH: &str = "기존 FFmpeg 실행 파일 (선택)";
pub const FFMPEG_HELP: &str = "절대 경로를 선택하면 버전을 확인하고 실행에 고정합니다. 빈칸이면 yt-dlp가 PATH에서 찾습니다. 외부 도구는 앱이 삭제하거나 업데이트하지 않습니다.";

pub const RECOVERY_CHECKING: &str =
    "이전 작업 상태를 확인하고 있습니다. 확인이 끝나면 시작할 수 있습니다.";
pub const RECOVERY_FAILED: &str = "이전 작업 상태를 확인하지 못했습니다. 앱 데이터 폴더 권한과 작업 DB를 확인한 뒤 다시 확인하세요.";
pub const RECOVERY_RETRY: &str = "이전 작업 다시 확인";
pub const RECOVERY_DONE: &str = "이전 작업 상태 재확인";
pub const RECOVERY_NO_AUTOSTART: &str = "중단된 다운로드를 자동으로 재시작하지 않습니다.";

pub const HISTORY: &str = "작업 이력";
pub const HISTORY_HELP: &str = "원래 옵션으로 재개합니다. 다운로드 화면에서 yt-dlp 경로를 확인하세요. 완료·취소된 작업과 옵션이 없는 이전 작업은 재개할 수 없습니다.";
pub const HISTORY_EMPTY: &str = "저장된 작업이 없습니다";
pub const HISTORY_LOADING: &str = "작업 이력을 읽는 중";
pub const HISTORY_FAILED: &str = "작업 이력을 읽지 못했습니다";
pub const HISTORY_REFRESH: &str = "처음부터 새로 고침";
pub const HISTORY_NEXT: &str = "다음 페이지";
pub const RESUME: &str = "재개";
pub const ATTEMPTS: &str = "시도";

pub const STATE_QUEUED: &str = "대기 중";
pub const STATE_PREPARING: &str = "준비 중";
pub const STATE_POSTPROCESSING: &str = "후처리 중";
pub const STATE_VERIFYING: &str = "파일 확인 중";
pub const STATE_STOPPING: &str = "중단 중";
pub const STATE_PAUSED: &str = "일시 중단됨";
pub const STATE_INTERRUPTED: &str = "비정상 중단됨";
pub const STATE_RETRY: &str = "재시도 대기 중";

pub const PAUSE: &str = "일시 중단";
pub const PAUSE_HELP: &str = "일시 중단은 프로세스를 종료하고 미완료 파일을 보존합니다. 이력에서 재개할 수 있지만 소스가 만료되면 실패할 수 있습니다. 취소한 작업은 재개하지 않습니다.";

pub const TOOLS: &str = "도구";
pub const TOOL_YT_DLP: &str = "yt-dlp";
pub const TOOL_DENO: &str = "Deno";
pub const TOOL_FFMPEG: &str = "FFmpeg·ffprobe";
pub const TOOL_RUNTIME_HELP: &str = "고정 버전을 앱 전용 폴더에 다운로드합니다. 설치 후 버전을 검증해 선택하고 다운로드에 사용하세요. 외부 도구와 영상은 제거하지 않습니다.";
pub const TOOL_RUNTIME_LICENSE: &str = "Deno 2.9.7은 공식 배포, FFmpeg·ffprobe는 BtbN LGPL 배포를 사용합니다. 크기·SHA256·고정 구성 파일을 검증합니다. 공개 배포용 공급 신뢰와 전체 법적 공지 검토는 진행 중입니다.";
pub const TOOL_EXECUTION_PENDING: &str = "관리 도구의 실행 종료를 확인하지 못했습니다. 파일 보호를 위해 관리 도구 실행과 변경을 보류합니다.";
pub const TOOL_RECOVERY_FAILED: &str = "이전 관리 도구 실행 기록을 안전하게 확인하지 못했습니다. 앱 전용 폴더의 권한을 확인하고 다시 시도하세요.";
pub const TOOL_SUPPLY_PENDING: &str = "자동 설치 준비 중: 공급 검증과 라이선스 자료를 확인하고 있습니다. 기존 관리 버전은 검증 후 선택할 수 있습니다.";
pub const TOOL_HELP: &str = "공식 yt-dlp 고정 버전을 앱 전용 폴더에 설치합니다. HTTPS·SHA256·버전을 검증한 뒤 활성화합니다. 외부 도구와 영상은 제거하지 않습니다.";
pub const TOOL_LICENSE: &str = "yt-dlp 2026.08.19: GPL-3.0-or-later와 해당 버전의 THIRD_PARTY_LICENSES 적용. FFmpeg·JS runtime은 위 도구 선택에서 각각 설치합니다. OS별 실행 검증은 개발 중입니다.";
pub const TOOL_UNSUPPORTED: &str =
    "이 OS/아키텍처의 고정 공급 파일은 아직 없습니다. 외부 yt-dlp 경로를 사용하세요.";
pub const TOOL_FAILED: &str = "도구 작업 실패·취소. 이전 활성 버전은 유지합니다. 경로·네트워크·공급 파일 검증 상태를 확인하세요.";
pub const TOOL_REFRESH: &str = "설치 목록 갱신";
pub const TOOL_INSTALL: &str = "고정 버전 설치";
pub const TOOL_INSTALL_YTDLP: &str = "yt-dlp 설치";
pub const TOOL_INSTALL_DENO: &str = "Deno 설치";
pub const TOOL_INSTALL_FFMPEG: &str = "FFmpeg·ffprobe 설치";
pub const TOOL_CANCEL: &str = "설치 중단 요청";
pub const TOOL_BUSY: &str =
    "설치·검증 작업 중. 중단 요청은 읽기 사이에 반영하며 최대 120초의 제한 안에서 응답합니다.";
pub const TOOL_RUNNING: &str = "다운로드 중에는 도구 설치·선택·제거를 할 수 없습니다.";
pub const TOOL_EMPTY: &str = "앱이 관리하는 설치가 없습니다";
pub const TOOL_ACTIVE: &str = "활성";
pub const TOOL_INACTIVE: &str = "보관됨";
pub const TOOL_SELECT: &str = "이 버전으로 다운로드";
pub const TOOL_REMOVE: &str = "보관 버전 제거";
pub const TOOL_USE: &str = "다운로드 설정에 적용";
pub const TOOL_USE_HELP: &str = "검사한 이 도구의 경로를 다운로드 설정에 넣습니다. 적용한 뒤 다운로드 화면에서 영상을 저장하세요.";
pub const TOOL_SELECT_HELP: &str =
    "이 버전의 파일과 실행 정보를 검사한 뒤 다운로드 도구로 선택합니다.";

pub const TOOL_HTTP_FAILED: &str =
    "도구 공급 서버에 연결하지 못했습니다. 네트워크를 확인하고 다시 시도하세요.";
pub const TOOL_HASH_FAILED: &str =
    "공급 파일의 크기 또는 SHA256이 일치하지 않습니다. 활성화하지 않았습니다.";
pub const TOOL_VERSION_FAILED: &str =
    "실행 파일 버전이 고정 manifest와 다릅니다. 활성화하지 않았습니다.";
pub const TOOL_CANCELLED: &str = "설치를 중단했습니다. 이전 활성 버전을 보존합니다.";
pub const TOOL_TIMEOUT: &str = "설치 시간 제한을 넘었습니다. 이전 활성 버전을 보존합니다.";
pub const TOOL_ACTIVE_REMOVE: &str =
    "활성 버전은 제거할 수 없습니다. 다른 관리 버전을 먼저 선택하세요.";
pub const TOOL_INVENTORY_LIMIT: &str = "관리 설치 목록 또는 디렉터리 수가 상한을 넘었습니다. 전체 목록을 확인할 수 없어 작업을 중단합니다.";

pub const TOOL_STAGING_RECOVERED: &str =
    "종료된 설치의 소유 증거를 확인해 임시 폴더를 정리했습니다. 폴더 수:";
pub const TOOL_STAGING_PRESERVED: &str = "실행 중이거나 소유·파일 증거를 확인할 수 없는 임시 폴더를 보존했습니다. 자동 삭제하지 않습니다. 폴더 수:";

pub const SUBTITLE_NONE: &str = "저장 안 함";
pub const SUBTITLE_SELECTED: &str = "선택 자막";
pub const SUBTITLE_AUTO_ALLOWED: &str = "자동 생성 자막 허용: 켜짐";
pub const SUBTITLE_AUTO_DENIED: &str = "자동 생성 자막 허용: 꺼짐";

pub const PROCESSING_SPEED: &str = "미디어 처리 배속";

pub const BACKGROUND: &str = "백그라운드 계속";
pub const BACKGROUND_RESTORE: &str =
    "창을 숨기고 작업을 유지합니다. dlpad를 다시 실행하면 돌아옵니다.";
pub const BACKGROUND_UNAVAILABLE: &str = "이 환경에서는 안전한 창 복귀를 준비하지 못했습니다.";

pub const OMIT_MPD: &str = "선택한 MPD 구조 제외";
pub const OMIT_ALL_MPD: &str = "MPD 구조 모두 제외";
pub const DIAGNOSTIC_FAILED: &str =
    "진단 자료 작업을 완료하지 못했습니다. 입력 파일과 기존 저장 파일은 유지됩니다.";
pub const DIAGNOSTIC_SAVED: &str = "선택한 새 파일로 저장했습니다.";
pub const PREVIEW_TOO_LARGE: &str = "미리보기는 2 MiB 이내로 제한됩니다. 로그나 첨부를 제외하세요.";
pub const DIAGNOSTIC_INPUT_TOO_LARGE: &str = "입력은 32 KiB 이내로 줄여 주세요.";

pub const APPEARANCE_FAILED: &str =
    "화면 설정을 불러오거나 저장하지 못했습니다. 기존 저장 자료는 유지됩니다.";

// Typed CPU download controls; shared with the existing domain option catalog.
pub const QUALITY_MODE: &str = "품질 선택";
pub const QUALITY_BEST: &str = "최고 품질";
pub const QUALITY_HEIGHT: &str = "영상 높이 제한";
pub const QUALITY_AUDIO: &str = "오디오 우선";
pub const QUALITY_HEIGHT_NUMBER: &str = "영상 높이는 144~4320 정수로 입력하세요.";
pub const QUALITY_HELP: &str = "최고 품질은 가능한 원본 품질을 선택합니다. 높이 제한은 원본보다 높은 화질을 만들지 않습니다. 오디오 우선은 오디오 트랙을 먼저 선택하고 없으면 영상 형식을 선택할 수 있습니다. 형식 변환은 수행하지 않습니다.";
pub const SUBTITLE_LANGUAGE: &str = "자막 언어";
pub const SUBTITLE_AUTO: &str = "자동 생성 자막 허용";

pub const REPORT_PREVIEW_PAGE: &str = "미리보기 페이지";
pub const REPORT_PREVIEW_PREVIOUS: &str = "이전 페이지";
pub const REPORT_PREVIEW_NEXT: &str = "다음 페이지";
pub const REPORT_PREVIEW_HELP: &str =
    "화면에는 한 페이지씩 표시합니다. 복사와 저장에는 전체 보고서가 포함됩니다.";
pub const PLAN_PREVIEW_INVALID: &str = "입력 옵션을 확인한 뒤 실행 계획을 확인하세요.";
pub const PLAN_PREVIEW_NON_UNICODE: &str =
    "문자로 표시할 수 없는 경로가 있어 미리보기를 제공할 수 없습니다.";
pub const PLAN_PREVIEW_TOO_LARGE: &str = "실행 계획 미리보기는 256 KiB 이내로 제한됩니다.";
pub const PLAN_PREVIEW_FAILED: &str = "실행 계획을 표시할 수 없습니다.";

pub const REPORT_PREVIEW_CONTENT: &str = "진단 자료 미리보기";
pub const PLAN_PREVIEW_CONTENT: &str = "실행 계획 내용";
pub const PLAN_PREVIEW_HELP: &str =
    "실행할 인수를 한 페이지씩 표시합니다. 원본 URL은 전체 생략합니다.";

// Session cookie source selection contains no cookie contents and is not persisted.
pub const COOKIE_SOURCE: &str = "이번 작업의 Netscape 쿠키 파일 (선택)";
pub const COOKIE_SESSION_HELP: &str = "원본은 보존하며 이번 작업에만 사용합니다. 재개할 때 다시 선택하세요. 인증 작업의 원시 로그는 비밀 보호를 위해 생략합니다.";

pub const QUALITY_HELP_TITLE: &str = "품질 선택 도움말";
pub const DEFAULT_QUALITY_HELP: &str = "0은 가능한 원본 최고 품질을 선택하고, 144~4320은 영상 높이의 상한입니다. 원본보다 높은 화질을 만들거나 형식을 변환하지 않습니다.";
pub const DEFAULT_QUALITY_DEFAULT_EXAMPLE: &str =
    "기본값: 0 (최고 품질) · 예: 1080 (영상 높이를 1080 이하로 제한)";
pub const DEFAULT_QUALITY_CONFLICT_VERSION: &str = "충돌: 숫자 한 개만 입력하며 이 화면은 오디오 우선 모드를 제공하지 않습니다. · 설치 버전: 검증되지 않음";
pub const QUALITY_DEFAULT_EXAMPLE: &str =
    "기본값: 최고 품질 · 예: 영상 높이 제한 1080 (144~4320), 또는 오디오 우선";
pub const QUALITY_CONFLICT_VERSION: &str = "충돌: 품질 모드는 하나만 선택하며 오디오 우선도 영상 형식으로 대체될 수 있습니다. 별도 형식 변환은 없습니다. · 설치 버전: 검증되지 않음";
pub const RANGE_HELP_TITLE: &str = "구간 입력 도움말";
pub const RANGE_HELP: &str = "시작과 끝을 함께 입력하여 해당 구간만 저장합니다. 끝은 시작보다 뒤여야 하며 밀리초까지 입력할 수 있습니다.";
pub const RANGE_DEFAULT_EXAMPLE: &str =
    "기본값: 시작과 끝을 모두 비우면 전체 영상 · 예: 시작 00:10:00.000, 끝 00:12:30.500";
pub const RANGE_CONFLICT_VERSION: &str =
    "충돌: 한쪽만 입력할 수 없으며 자막 저장과 함께 사용할 수 없습니다. · 설치 버전: 검증되지 않음";
pub const CUT_HELP_TITLE: &str = "자르기 방식 도움말";
pub const CUT_DEFAULT_EXAMPLE: &str = "기본값: 빠르게 (키프레임 기준) · 예: 구간의 시작 위치를 더 정확히 맞추려면 정확하게 (재인코딩) 선택";
pub const CUT_CONFLICT_VERSION: &str = "충돌: 구간을 입력했을 때만 적용하며 전체 영상에서는 선택해도 자르기 옵션을 사용하지 않습니다. 구간과 자막 저장은 함께 사용할 수 없습니다. · 설치 버전: 검증되지 않음";
pub const FFMPEG_HELP_TITLE: &str = "FFmpeg 선택 도움말";
pub const FFMPEG_DEFAULT_EXAMPLE: &str = "기본값: 빈칸, yt-dlp의 PATH 탐색 · 예: 기존 실행 파일의 절대 경로 /opt/ffmpeg/bin/ffmpeg 또는 C:\\Tools\\ffmpeg\\bin\\ffmpeg.exe";
pub const FFMPEG_CONFLICT_VERSION: &str = "충돌: 앱이 실행 계획의 FFmpeg 위치를 관리하며 고급 옵션으로 중복 지정할 수 없습니다. · 설치 버전별 기능: 검증되지 않음. 선택한 파일의 버전은 실행 시 확인합니다.";
pub const COOKIE_HELP_TITLE: &str = "세션 쿠키 선택 도움말";
pub const COOKIE_DEFAULT_EXAMPLE: &str = "기본값: 선택 안 함 (쿠키 없이 실행) · 예: 본인이 내보낸 Netscape 형식 파일의 절대 경로 /tmp/session.cookies 또는 C:\\Temp\\session.cookies";
pub const COOKIE_CONFLICT_VERSION: &str = "충돌: 브라우저 로그인·브라우저 쿠키 자동 추출은 제공하지 않습니다. 앱의 전용 세션 복사본만 사용하며 재개에는 새 선택이 필요합니다. 안전한 권한 검증을 지원하지 않는 OS에서는 사용할 수 없습니다. · 설치 버전: 검증되지 않음";
pub const COOKIE_IMPORT_FAILED: &str =
    "쿠키 파일을 안전하게 가져오지 못했습니다. 파일 형식과 앱 전용 폴더 권한을 확인하세요.";
pub const COOKIE_UNAVAILABLE: &str = "이 OS의 안전한 쿠키 파일 권한 검증은 아직 제공되지 않습니다.";
pub const COOKIE_CLEANUP_FAILED: &str = "쿠키 복사본의 소유권이 변경되어 자동 정리를 보류했습니다.";

pub const COOKIE_RECOVERY_PENDING: &str = "이전 쿠키 복사본의 종료·소유권 확인이 남아 안전하게 보존했습니다. 새 작업은 새 쿠키 선택으로 실행합니다.";
pub const COOKIE_RECOVERY_FAILED: &str = "이전 쿠키 복사본을 안전하게 확인하지 못했습니다. 앱 전용 폴더의 권한을 확인하고 다시 시도하세요.";

pub const DENO_PATH: &str = "Deno 실행 파일 (선택 사항)";
pub const DENO_HELP: &str = "선택한 Deno의 버전을 실행 전에 확인하고 이 작업에 사용합니다. 실행 중에는 선택을 변경할 수 없습니다.";
pub const DENO_HELP_TITLE: &str = "Deno 선택 도움말";
pub const DENO_DEFAULT_EXAMPLE: &str = "기본값: 빈칸, yt-dlp의 기본 runtime 탐색 · 예: 기존 Deno 실행 파일의 절대 경로 /opt/deno/deno 또는 C:/Tools/deno/deno.exe";
pub const DENO_CONFLICT_VERSION: &str = "선택하면 이 Deno만 사용합니다. 고급 옵션으로 중복 지정할 수 없습니다. 설치 버전과 사이트 호환성: 검증되지 않음. 외부 설치 파일은 앱에서 삭제하지 않습니다.";

pub const APP_PURPOSE: &str = "영상 저장과 파일 관리";
pub const DOWNLOAD_ENVIRONMENT: &str = "다운로드 도구 및 인증 설정";
pub const OPTIONAL_MEDIA: &str = "구간 및 자막";

pub const INPUT_CHECK: &str = "한글 입력 확인";

pub const QUALITY_SUMMARY: &str =
    "기본은 최고 품질입니다. 필요하면 해상도 제한이나 오디오 우선으로 바꿀 수 있습니다.";
pub const TOOL_SETUP_HINT: &str = "도구 화면에서 설치한 도구를 다운로드에 사용하세요.";

pub const CHOOSE_FOLDER: &str = "폴더 선택…";
pub const CHOOSE_SAVE_FOLDER: &str = "저장 폴더 선택";
pub const FOLDER_PICKER_FAILED: &str = "폴더를 선택하지 못했습니다. 기존 저장 위치는 유지됩니다.";
pub const FOLDER_PICKER_UNAVAILABLE: &str =
    "이 실행 환경에서는 폴더 선택 창을 사용할 수 없습니다. 경로를 직접 입력하세요.";
pub const FOLDER_PICKER_BUSY: &str = "폴더 선택 창에서 위치를 선택하거나 취소해 주세요.";
pub const LIBRARY_LOADING: &str = "파일 목록을 확인하고 있습니다…";
pub const LIBRARY_EMPTY: &str = "표시할 파일이 없습니다";
pub const LIBRARY_EMPTY_HELP: &str =
    "설정의 저장 위치에서 기본 폴더를 선택하세요. 이미 선택했다면 목록 새로고침을 누르세요.";
pub const LIBRARY_RETRY: &str = "저장 폴더와 접근 권한을 확인한 뒤 다시 새로고침하세요.";
pub const TOOL_DETAILS: &str = "버전·라이선스 안내";
pub const TOOL_EXECUTABLE_DETAILS: &str = "실행 파일 경로";
pub const TOOL_EXECUTABLE_PATH: &str = "활성 도구 실행 파일";
pub const TOOL_EXECUTABLE_COPY: &str = "경로 복사";

pub const THEME_HELP: &str = "앱의 색상을 선택합니다. 시스템 설정을 따를 수도 있습니다.";
pub const SCALE_HELP: &str = "글자와 조작 영역의 크기를 함께 조절합니다.";
pub const NETWORK_OBSERVATION: &str = "네트워크 관측 설정";

// Media workspace vocabulary, shared by preparation, preview and timeline.
pub const MEDIA_SCREEN_TITLE: &str = "영상 준비";
pub const MEDIA_SOURCE_HINT: &str =
    "영상 URL은 입력 후 자동 확인합니다. 영상 파일은 이곳에 끌어 놓으세요.";
pub const MEDIA_PREPARE: &str = "영상 확인";
pub const MEDIA_PREVIEW: &str = "영상 미리보기";
pub const MEDIA_EMPTY: &str = "저장할 영상의 내용을 먼저 확인하세요";
pub const MEDIA_RANGE: &str = "저장 구간";
pub const MEDIA_PLAYHEAD: &str = "미리보기 위치";
pub const MEDIA_PLAY: &str = "미리보기 재생";
pub const MEDIA_PAUSE: &str = "미리보기 일시 정지";
pub const MEDIA_ALL: &str = "전체 구간";
pub const MEDIA_WAIT: &str = "영상 정보를 확인하고 있습니다";
pub const MEDIA_FRAME_WAIT: &str = "선택한 위치를 불러오고 있습니다";
pub const MEDIA_FAILED: &str = "영상을 확인하지 못했습니다. 주소와 미리보기 도구를 확인해 주세요.";
pub const MEDIA_INVALID_DATA: &str =
    "미리보기 데이터가 올바르지 않습니다. 영상을 다시 확인해 주세요.";
pub const MEDIA_DATA_LIMIT: &str =
    "미리보기 데이터가 허용 범위를 넘었습니다. 영상을 다시 확인해 주세요.";
pub const MEDIA_TOOL_FAILED: &str =
    "미리보기 도구 실행에 실패했습니다. 도구와 연결을 확인한 뒤 다시 시도하세요.";
pub const MEDIA_UNSUPPORTED: &str =
    "이 영상은 미리보기를 지원하지 않습니다. 다른 영상을 선택해 주세요.";
pub const MEDIA_TOOLS: &str = "미리보기에 yt-dlp와 FFmpeg가 필요합니다.";
pub const MEDIA_CANCELLED: &str = "영상 확인을 취소했습니다";
pub const MEDIA_LOCAL: &str = "로컬 영상";
pub const MEDIA_REMOTE: &str = "영상 URL";
pub const MEDIA_LOCAL_HINT: &str = "로컬 영상은 원본을 변경하지 않고 내용과 구간을 확인합니다.";
pub const MEDIA_PREVIEW_SILENT: &str = "저해상도 무음 미리보기";
pub const MEDIA_DROP_ONE: &str = "영상 파일 한 개만 끌어 놓아 주세요.";
pub const MEDIA_RANGE_INVALID: &str = "시작과 끝 시간을 영상 길이 안에서 입력하세요.";
pub const MEDIA_SAVE_SETTINGS: &str = "저장 설정";
pub const MEDIA_URL_INVALID: &str = "올바른 HTTP 또는 HTTPS 영상 주소를 입력해 주세요.";

pub const MEDIA_CLEANUP_FAILED: &str =
    "미리보기 작업의 종료를 확인하지 못했습니다. 새 작업을 중지했습니다.";

pub const MEDIA_SKIP_PREVIEW: &str = "미리보기 없이 다운로드";
pub const MEDIA_URL_REQUIRED: &str = "영상 URL을 입력하세요.";
pub const MEDIA_LOCATION_REQUIRED: &str = "저장 폴더를 선택하세요.";
pub const MEDIA_TOOL_REQUIRED: &str = "다운로드에 yt-dlp가 필요합니다.";
pub const MEDIA_PREVIEW_REQUIRED: &str = "영상을 먼저 확인하세요.";
pub const MEDIA_TIMEOUT: &str = "영상 확인 시간이 초과됐습니다. 연결을 확인한 뒤 다시 시도하세요.";

pub const MEDIA_INFO: &str = "영상 정보";
pub const MEDIA_SELECTED_DURATION: &str = "선택 길이";

pub const MEDIA_FRAME_UNAVAILABLE: &str =
    "이 위치의 프레임을 표시할 수 없습니다. 다른 위치를 선택하세요.";

pub const INSTALL_TOOL_OK: &str = "설치됨, 앱의 도구 화면에서 선택 후 사용";
pub const INSTALL_TOOL_NOT_STARTED: &str = "시작하지 않음";
pub const INSTALL_TOOL_SUMMARY: &str = "선택 도구";

pub const LIBRARY_PREVIEW: &str = "선택한 영상 확인";
pub const LIBRARY_SELECT_HELP: &str =
    "파일을 선택한 뒤 영상 확인을 누르세요. 원본은 변경하지 않습니다.";
pub const MEDIA_ZOOM_SELECTION: &str = "선택 주변 확대";
pub const MEDIA_SHOW_ALL: &str = "전체 보기";
pub const MEDIA_FULL_CONTEXT: &str = "전체 영상";
pub const MEDIA_VISIBLE_WINDOW: &str = "표시 구간";

pub const LOCAL_EXPORT_SAVE_SELECTION: &str = "선택 구간 저장";
pub const LOCAL_EXPORT_RUNNING: &str = "선택한 구간을 새 파일로 저장하고 있습니다.";
pub const LOCAL_EXPORT_CANCEL: &str = "구간 저장 취소";
pub const LOCAL_EXPORT_WAITING_ACK: &str = "구간 저장 작업의 종료를 확인하고 있습니다.";
pub const LOCAL_EXPORT_SAVED: &str = "선택한 구간을 새 파일로 저장했습니다.";
pub const LOCAL_EXPORT_RETAINED_STAGE: &str = "검증용 임시 링크도 보존했습니다.";
pub const LOCAL_EXPORT_RETAINED_STAGE_EXPLANATION: &str = "새 폴더에 구간 파일을 저장합니다. 검증용 임시 링크는 같은 파일을 가리키며 자동 삭제하지 않습니다. 개발 후보는 MPEG-4·AAC, 최대 10분·결과 256 MiB를 지원합니다.";
pub const LOCAL_EXPORT_FAILED_PRESERVED: &str =
    "구간 저장을 완료하지 못했습니다. 원본과 생성된 파일은 보존됩니다.";
pub const LOCAL_EXPORT_PARTIAL_STAGE: &str = "일부 출력이 남아 있을 수 있습니다.";
pub const LOCAL_EXPORT_FINAL_PRESENT_ON_FAILURE: &str =
    "최종 파일 이름은 생성되었습니다. 완료 여부를 확인해야 합니다.";
pub const LOCAL_EXPORT_CLEANUP_UNCERTAIN: &str =
    "작업 종료 확인이 남아 새 작업과 정상 종료를 중단했습니다.";
pub const LOCAL_EXPORT_FILE_VERIFIED_CLEANUP_PENDING: &str =
    "저장 파일 검증은 통과했지만 작업 종료 확인이 남았습니다.";
pub const LOCAL_EXPORT_MUTATION_UNKNOWN: &str =
    "작업이 중단되어 생성된 파일 상태를 확인하지 못했습니다.";
pub const LOCAL_EXPORT_BLOCKED: &str = "진행 중인 작업과 도구 상태를 먼저 확인하세요.";
pub const LOCAL_EXPORT_PREVIEW_BUSY: &str = "미리보기 처리가 끝난 뒤 구간을 저장할 수 있습니다.";
pub const LOCAL_EXPORT_LOCAL_REQUIRED: &str = "먼저 로컬 영상을 확인하고 유효한 구간을 선택하세요.";
pub const LOCAL_EXPORT_UNAVAILABLE: &str = "이 OS에서는 로컬 구간 저장을 아직 제공하지 않습니다.";
pub const LOCAL_EXPORT_START_FAILED: &str = "구간 저장 작업을 시작하지 못했습니다.";
pub const LOCAL_EXPORT_RANGE_LIMIT: &str =
    "개발 후보에서는 한 번에 최대 10분 구간을 저장할 수 있습니다.";

pub const LOCAL_EXPORT_TOOL_REQUIRED: &str =
    "FFmpeg 실행 파일을 지정하세요. 같은 폴더의 ffprobe도 필요합니다.";

pub const MEDIA_CHANGE_SOURCE: &str = "다른 영상 열기";

pub const RANGE_START_LABEL: &str = "시작 시간";
pub const RANGE_END_LABEL: &str = "끝 시간";

pub const TOOL_PREF_HELP: &str = "직접 지정한 경로를 기억하거나, 앱에서 선택한 도구를 사용합니다. 시작 환경 설정이 있으면 재실행할 때 우선 적용합니다. 저장한 경로도 사용할 때 다시 확인합니다.";
pub const TOOL_PREF_MANUAL: &str = "입력한 경로 기억";
pub const TOOL_PREF_AUTO: &str = "자동 선택";
pub const TOOL_PREF_SAVE: &str = "도구 선택 저장";
pub const TOOL_PREF_SAVING: &str = "도구 선택을 저장하고 있습니다.";
pub const TOOL_PREF_SAVED: &str = "도구 선택을 저장했습니다.";
pub const TOOL_PREF_INVALID: &str =
    "기억할 경로는 절대 경로로 입력하세요. 이전 저장값은 유지됩니다.";
pub const TOOL_PREF_FAILED: &str = "도구 선택을 저장하지 못했습니다. 이전 저장값은 유지됩니다.";
pub const TOOL_PREF_LOAD_FAILED: &str =
    "저장한 도구 선택을 읽지 못했습니다. 자동으로 덮어쓰지 않습니다.";
pub const TOOL_PREF_SESSION_ONLY: &str = "이번 실행에서만 적용했습니다. 이전 저장값은 유지됩니다.";
pub const TOOL_PREF_LOADING: &str = "저장한 도구 선택을 읽고 있습니다.";
pub const TOOL_PREF_READY: &str = "변경한 도구 선택은 저장을 눌러 기억합니다.";
pub const TOOL_PREF_DIRTY: &str = "변경한 도구 선택을 아직 저장하지 않았습니다.";
pub const TOOL_PREF_OLDER_SAVED: &str =
    "앞서 요청한 선택은 저장했습니다. 현재 변경 내용은 아직 저장하지 않았습니다.";

pub const TOOL_EXECUTABLE_FILTER: &str = "실행 파일 (*.exe)";
pub const TOOL_CHOOSE_EXECUTABLE: &str = "기존 도구 실행 파일 선택";
pub const TOOL_BROWSE_HELP: &str = "기존 실행 파일을 선택합니다. 선택한 경로는 저장을 눌러 기억하며 앱이 설치하거나 제거하지 않습니다.";
pub const TOOL_PICKER_FAILED: &str =
    "실행 파일을 선택하지 못했습니다. 기존 경로를 유지하고 직접 입력하거나 다시 선택하세요.";

pub const TOOL_PICKER_BUSY: &str = "실행 파일 선택 창에서 파일을 고르거나 취소하세요.";

pub const TOOL_BROWSE_YTDLP: &str = "yt-dlp 선택…";
pub const TOOL_BROWSE_FFMPEG: &str = "FFmpeg 선택…";
pub const TOOL_BROWSE_DENO: &str = "Deno 선택…";

pub const HISTORY_RESUME_DETAILS: &str = "재개 안내";
pub const HISTORY_EMPTY_HELP: &str =
    "다운로드 탭에서 영상 URL을 입력하고 다운로드를 시작하세요. 저장한 작업은 여기에 표시됩니다.";
pub const HISTORY_RETRY_HELP: &str =
    "새로 고침으로 다시 읽어 보세요. 저장된 작업은 그대로 보존합니다.";
pub const HISTORY_RESUME_INSTRUCTION: &str = "도구 경로를 확인한 뒤 원래 옵션으로 재개하세요.";

pub const MEDIA_PREVIEW_FFMPEG_REQUIRED: &str = "영상 확인에 FFmpeg가 필요합니다.";
pub const MEDIA_OPEN_TOOLS: &str = "필요한 도구 설정…";
pub const MEDIA_OPEN_YTDLP: &str = "yt-dlp 설정…";
pub const MEDIA_OPEN_FFMPEG: &str = "FFmpeg 설정…";

// Tool selection: user purpose and costs first; verification details are expanded.
pub const TOOL_YTDLP_ROLE: &str = "영상 주소의 정보를 확인하고 다운로드합니다.";
pub const TOOL_DENO_ROLE: &str =
    "일부 사이트의 영상 확인과 다운로드에 필요한 JavaScript 실행 환경입니다.";
pub const TOOL_FFMPEG_ROLE: &str = "영상·음성을 합치고, 영상 확인과 선택 구간 저장에 사용합니다.";
pub const TOOL_YTDLP_ORIGIN_MANAGED: &str = "yt-dlp 공식 배포 · 앱 전용 폴더에 설치";
pub const TOOL_DENO_ORIGIN_MANAGED: &str = "Deno 공식 배포 · 앱 전용 폴더에 설치";
pub const TOOL_FFMPEG_ORIGIN_MANAGED: &str = "BtbN LGPL 배포 · 앱 전용 폴더에 설치";
pub const TOOL_DOWNLOAD_SIZE_LABEL: &str = "다운로드";
pub const TOOL_INSTALL_FILES_SIZE_LABEL: &str = "설치 파일";
pub const TOOL_VERSION_LABEL: &str = "버전";
pub const TOOL_PLATFORM_LABEL: &str = "대상 플랫폼";
pub const TOOL_SOURCE_URL_LABEL: &str = "배포 주소";
pub const TOOL_SHA256_LABEL: &str = "SHA256";
pub const TOOL_LICENSE_LABEL: &str = "라이선스";
pub const TOOL_VERIFICATION_DETAILS: &str =
    "크기·SHA256·버전을 확인한 파일만 설치합니다. 설치한 버전은 목록에서 선택할 수 있습니다.";
pub const TOOL_DISTRIBUTION_REVIEW_STATUS: &str = "외부 도구와 영상은 제거하지 않습니다. 공개 배포용 공급 신뢰와 전체 법적 공지 검토는 진행 중입니다.";

pub const TOOL_DENO_LICENSE_SUMMARY: &str =
    "Deno 고정 배포의 MIT 라이선스가 적용됩니다. 관련 공지의 전체 검토는 진행 중입니다.";
pub const TOOL_FFMPEG_LICENSE_SUMMARY: &str = "FFmpeg·ffprobe 고정 배포의 LGPL-3.0-or-later 라이선스가 적용됩니다. 관련 공지의 전체 검토는 진행 중입니다.";

// Access to the existing reason for a disabled download.
pub const START_CONDITIONS: &str = "시작 조건 확인";

pub const DIAGNOSTIC_MPD_DETAILS: &str = "MPD 구조 첨부·제외 (선택)";

pub const DIAGNOSTIC_CHOOSE_EXPORT: &str = "진단 자료 저장 위치 선택";
pub const DIAGNOSTIC_CHOOSE_MPD: &str = "추가할 MPD 파일 선택";
pub const DIAGNOSTIC_BROWSE_EXPORT: &str = "저장 위치 선택…";
pub const DIAGNOSTIC_BROWSE_MPD: &str = "MPD 파일 선택…";
pub const DIAGNOSTIC_PICKER_BUSY: &str = "파일 선택 창에서 위치나 파일을 선택하거나 취소해 주세요.";
pub const DIAGNOSTIC_PICKER_FAILED: &str = "파일을 선택하지 못했습니다. 기존 입력은 유지됩니다.";
pub const DIAGNOSTIC_PICKER_UNAVAILABLE: &str =
    "이 실행 환경에서는 파일 선택 창을 사용할 수 없습니다. 경로를 직접 입력하세요.";
pub const REPORT_MARKDOWN_FILTER: &str = "Markdown 진단 자료";
pub const REPORT_JSON_FILTER: &str = "JSON 진단 자료";
pub const REPORT_MPD_FILTER: &str = "MPD 파일";

pub const ADVANCED_OPTION_DETAILS: &str = "설명·예시 보기";

pub const LIBRARY_PATH_DETAIL_ACTION: &str = "선택한 파일 경로 확인";
pub const LIBRARY_PATH_DETAIL_TITLE: &str = "전체 파일 경로";
pub const LIBRARY_PATH_DETAIL_KEYS: &str =
    "PageUp·PageDown으로 읽고 Home·End로 처음과 끝으로 이동합니다. Esc로 닫습니다.";
pub const LIBRARY_PATH_DETAIL_CLOSE: &str = "닫기";

pub const ADVANCED_RESET: &str = "기본값으로 되돌리기";

pub const RATE_BPS: &str = "B/s";
pub const RATE_KBPS: &str = "KB/s";
pub const RATE_MBPS: &str = "MB/s";
pub const RATE_GBPS: &str = "GB/s";
pub const RATE_UNLIMITED: &str = "제한 없음";
pub const RATE_PRESET_ONE: &str = "1 MB/s";
pub const RATE_PRESET_FIVE: &str = "5 MB/s";
pub const RATE_PRESET_TEN: &str = "10 MB/s";
pub const RATE_UNIT_LABEL: &str = "다운로드 속도 단위";
pub const RATE_UNIT_HELP: &str =
    "1 MB/s = 1,000 KB/s. 소수점은 점(.)을 사용하세요. 비우면 제한하지 않습니다.";
pub const RATE_NUMBER_ERROR: &str = "0보다 크고 1,000 GB/s 이하인 속도를 입력하세요. 소수점은 점(.)을 사용하며 1 B/s보다 작은 단위는 지원하지 않습니다.";

pub const RATE_CLI_EXAMPLE_HELP: &str =
    "도구 인수의 1048576은 B/s 값이며, 이 입력에서는 1.048576 MB/s와 같습니다.";

pub const NETWORK_CONNECTION: &str = "관측 연결";
pub const NETWORK_MANUAL_HELP: &str = "이름을 직접 입력할 때 적용하세요. 빈칸은 관측을 끕니다.";
pub const NETWORK_AUTOMATIC: &str = "자동";
pub const NETWORK_OFF: &str = "끄기";
pub const NETWORK_REFRESH: &str = "목록 새로고침";
pub const NETWORK_SELECTED: &str = "선택한 연결";
pub const NETWORK_AUTOMATIC_HELP: &str =
    "자동은 기본 연결을 관측합니다. 다른 앱 트래픽도 포함합니다.";
pub const NETWORK_MANUAL_FAILED: &str =
    "선택한 연결을 관측하지 못했습니다. 목록을 새로고침하거나 다른 연결을 선택하세요.";
pub const NETWORK_LIST_FAILED: &str = "연결 목록을 읽지 못했습니다. 새로고침하세요.";
pub const NETWORK_NO_ACTIVE: &str = "관측할 활성 연결 없음";
pub const NETWORK_AUTOMATIC_FAILED: &str = "네트워크 자동 선택 실패";
pub const NETWORK_OFF_STATUS: &str = "네트워크 관측 꺼짐";

pub const TOOL_AUTO_CHECKING: &str = "설치된 도구를 확인하고 있습니다.";
pub const TOOL_AUTO_UNAVAILABLE: &str = "사용 가능한 설치 도구가 없습니다. 도구 탭에서 설치하세요.";

pub const TOOL_VERIFY_SELECT: &str = "검사 후 사용";

// Candidate constants for root centralized catalog only. Not a standalone/live catalog.
pub const TOOL_UPDATE_TITLE: &str = "버전 확인";
pub const TOOL_UPDATE_CURRENT: &str = "현재 사용 중";
pub const TOOL_UPDATE_CURRENT_UNKNOWN: &str = "아직 확인하지 않음";
pub const TOOL_UPDATE_EXTERNAL: &str = "직접 지정한 도구 — 버전 미확인";
pub const TOOL_UPDATE_OBSERVED: &str = "확인한 배포 버전";
pub const TOOL_UPDATE_NOT_CHECKED: &str = "아직 확인하지 않음";
pub const TOOL_UPDATE_CHECK: &str = "새 버전 확인";
pub const TOOL_UPDATE_INSTALL: &str = "확인한 버전 설치";
pub const TOOL_UPDATE_INSTALLED: &str = "보관된 버전";
pub const TOOL_UPDATE_CHOOSE: &str = "버전 선택";
pub const TOOL_UPDATE_USE_SELECTED: &str = "선택한 버전 사용";
pub const TOOL_UPDATE_INSTALL_ONLY_HELP: &str =
    "설치해도 현재 사용 중인 버전은 유지됩니다. 설치한 버전은 아래에서 선택해 사용하세요.";
pub const TOOL_UPDATE_USE_HELP: &str =
    "선택한 설치의 파일과 실행 버전을 다시 검증한 뒤 사용 버전을 바꿉니다.";
pub const TOOL_UPDATE_SELECT_FIRST: &str = "사용할 보관 버전을 먼저 선택하세요.";
pub const TOOL_UPDATE_CHECKING: &str = "배포 버전을 확인하고 있습니다.";
pub const TOOL_UPDATE_INSTALLING: &str =
    "설치 파일과 버전을 검증하고 있습니다. 현재 사용 버전은 유지됩니다.";
pub const TOOL_UPDATE_SELECTING: &str = "선택한 설치를 검증하고 있습니다.";
pub const TOOL_UPDATE_OFFLINE: &str = "배포 정보를 읽지 못했습니다. 네트워크를 확인하고 새 버전 확인을 다시 누르세요. 보관된 버전은 따로 선택할 수 있습니다.";
pub const TOOL_UPDATE_UNSUPPORTED: &str =
    "이 도구의 버전 확인·설치는 현재 OS에서 지원하지 않습니다. 기존 경로는 유지합니다.";
pub const TOOL_UPDATE_UNVERIFIED: &str =
    "이 버전의 설치 준비를 확인하지 못했습니다. 확인한 배포 버전만으로 설치하지 않습니다.";
pub const TOOL_UPDATE_INVALID_SNAPSHOT: &str =
    "버전 목록을 안전하게 표시하지 못했습니다. 설치 목록을 다시 읽어 주세요.";
pub const TOOL_UPDATE_ALREADY_INSTALLED: &str =
    "이 버전은 이미 보관되어 있습니다. 아래에서 선택해 사용하세요.";
pub const TOOL_UPDATE_VERSION_ACTIVE: &str = "이미 사용 중인 버전입니다.";
pub const TOOL_UPDATE_NO_INSTALLED: &str = "보관된 관리 도구 버전이 없습니다.";

// Root-owned central catalog additions (candidate only).
pub const MEDIA_FULL_RANGE_SELECTED: &str = "전체 구간 선택됨";
pub const MEDIA_DURATION: &str = "영상 길이";
pub const MEDIA_OVERVIEW_CONTEXT: &str = "전체 영상에서 현재 표시 위치";
pub const MEDIA_OVERVIEW_HELP: &str =
    "작은 막대는 전체 영상 안에서 현재 보고 있는 시간축의 위치입니다.";
pub const MEDIA_RANGE_TIMELINE_HELP: &str =
    "막대 클릭은 미리보기 이동, 시작·끝 손잡이는 구간 조절입니다. 시간도 직접 입력할 수 있습니다.";
pub const MEDIA_TIMELINE_SEEK_HELP: &str =
    "막대를 클릭하거나 끌어 미리보기 위치를 이동합니다. 저장 구간은 바뀌지 않습니다.";
pub const MEDIA_RANGE_START_HANDLE_HELP: &str = "시작 손잡이를 좌우로 끌어 저장 시작을 조절합니다.";
pub const MEDIA_RANGE_END_HANDLE_HELP: &str = "끝 손잡이를 좌우로 끌어 저장 끝을 조절합니다.";
// Optional wording replacements, retain existing constants/handlers and IDs:
// MEDIA_SHOW_ALL = "전체 시간축 보기" (viewport only)
// MEDIA_ZOOM_SELECTION = "선택 구간 확대" (viewport only)
// MEDIA_ALL = "구간 초기화" (existing full-range/reset action, keep enabled to repair invalid drafts)

// Root central catalog only; no live edits by candidate.
pub const TOOL_CACHE_REFRESHING: &str = "최근 목록을 표시하며 다시 확인하고 있습니다.";
pub const TOOL_CACHE_WAITING: &str = "최근 목록을 표시합니다. 진행 중인 확인이 끝나면 갱신합니다.";
pub const TOOL_CACHE_STALE: &str = "최근에 확인한 목록입니다. 사용·제거 전에 다시 검증합니다.";
pub const TOOL_CACHE_FAILED: &str = "목록을 다시 확인하지 못했습니다. 최근 목록을 유지합니다.";

// Root integrates catalog. Required inline statuses are never tooltip-only.
pub const LIBRARY_OPEN_DEFAULT: &str = "기본 앱으로 열기";
pub const LIBRARY_OPEN_FOLDER: &str = "폴더 열기";
pub const LIBRARY_SELECTED_FILE: &str = "선택 파일:";
pub const LIBRARY_FILE_ACTION_HELP: &str =
    "더블클릭이나 Enter로 앱 안에서 미리봅니다. 기본 앱 열기는 별도 동작입니다.";
pub const LIBRARY_OPEN_UNSUPPORTED: &str =
    "이 환경에서는 기본 앱이나 폴더를 열 수 없습니다. 파일 경로를 확인해 주세요.";
pub const LIBRARY_OPEN_EXTENSION: &str =
    "이 파일 형식은 기본 앱으로 열 수 없습니다. 폴더에서 파일을 확인해 주세요.";
pub const LIBRARY_OPEN_NO_ASSOCIATION: &str =
    "이 파일을 열 기본 앱이 없습니다. 폴더에서 파일을 확인하거나 Windows 기본 앱을 설정해 주세요.";
pub const LIBRARY_OPEN_CANCELLED: &str = "열기 요청을 보내기 전에 취소했습니다.";
pub const LIBRARY_OPEN_TIMEOUT: &str =
    "열기 준비가 오래 걸려 요청을 보내지 않았습니다. 다시 선택해 주세요.";
pub const LIBRARY_OPEN_AUTOMATION_BLOCKED: &str =
    "분리된 시험 화면에서는 기본 앱을 실행하지 않습니다.";
pub const LIBRARY_OPEN_CHANGED: &str =
    "파일이 바뀌었거나 원본을 확인할 수 없습니다. 새로고침 후 다시 선택해 주세요.";
pub const LIBRARY_OPEN_FAILED: &str =
    "기본 앱이나 폴더를 열지 못했습니다. 파일 경로와 기본 앱 설정을 확인해 주세요.";
pub const LIBRARY_OPEN_HANDED_OFF: &str =
    "Windows에 열기를 요청했습니다. 기본 앱이나 폴더 창을 확인해 주세요.";
pub const LIBRARY_OPEN_HANDED_OFF_LATE: &str =
    "Windows에 이미 열기를 요청했습니다. 늦은 취소로 열린 앱을 닫지는 않습니다.";
pub const LIBRARY_OPEN_HANDED_OFF_CHANGED: &str =
    "Windows에 열기를 요청한 뒤 파일이 바뀌었습니다. 열린 앱과 원본을 확인해 주세요.";

pub const LIBRARY_MODIFIED: &str = "수정 시각";

pub const LIBRARY_MODIFIED_UNKNOWN: &str = "알 수 없음";

pub const TOOL_UPDATE_BUSY_BOUND: &str =
    "도구 작업 중입니다. 새 버전 확인과 설치는 최대 160초가 걸릴 수 있습니다.";

pub const TOOL_UPDATE_EVIDENCE_FAILED: &str =
    "설치를 완료하지 못했습니다. 확인 자료를 보관했습니다.";

pub const TOOL_UPDATE_EVIDENCE_INCOMPLETE: &str =
    "새 버전은 설치됐지만 기록 저장을 완료하지 못했습니다. 현재 사용하는 버전은 바뀌지 않았습니다.";

pub const TOOL_UPDATE_EVIDENCE_INSTALLED: &str =
    "새 버전을 설치하고 기록을 보관했습니다. 사용할 버전을 선택해 주세요.";

pub const TOOL_UPDATE_EVIDENCE_PATH: &str = "설치 기록 위치";

pub const TOOL_UPDATE_EVIDENCE_PREPARED: &str =
    "설치 자료를 보관했습니다. 설치 완료 여부는 확인되지 않았습니다.";

pub const TOOL_UPDATE_RECEIPT_FULL: &str =
    "설치 기록 저장 공간이 가득 찼습니다. 새 설치를 시작할 수 없습니다.";

pub const TOOL_UPDATE_RECEIPT_UNAVAILABLE: &str = "설치 기록을 안전하게 저장할 수 없습니다. 보관된 버전을 사용하거나 도구 경로를 직접 지정해 주세요.";

pub const TOOL_UPDATE_SELECTION_CHECKING: &str =
    "사용할 수 있는지 확인하고 있습니다. 잠시 기다려 주세요.";

pub const PRESERVE_VIDEO_TITLE: &str = "파일명에 영상 제목 유지";
pub const PRESERVE_VIDEO_TITLE_HELP: &str = "제목 뒤에 작업 식별자를 붙여 같은 제목의 영상을 구분합니다. 파일명에 쓸 수 없는 문자는 정리하며, 기존 파일은 덮어쓰지 않습니다. 이전 작업을 재개하면 당시 파일명 설정을 유지합니다.";
pub const PRESERVE_VIDEO_TITLE_EXAMPLE: &str = "예: 영상 제목 [작업 식별자].mp4 · 기본 켜짐";

// Root catalog integration only; no live catalog changed.
pub const LIBRARY_EXPORT_ACTION: &str = "복사 내보내기…";
pub const LIBRARY_EXPORT_TITLE: &str = "새 이름으로 복사 내보내기";
pub const LIBRARY_EXPORT_PRESERVE: &str =
    "원본은 유지합니다. 선택한 폴더에 새 파일을 만들며 기존 파일은 덮어쓰지 않습니다.";
pub const LIBRARY_EXPORT_FALLBACK: &str = "영상";
pub const LIBRARY_EXPORT_COPY_SUFFIX: &str = "복사";
pub const LIBRARY_EXPORT_STALE: &str =
    "선택 파일이나 저장 폴더가 바뀌었습니다. 현재 파일을 선택하고 내보내기를 다시 열어 주세요.";
pub const LIBRARY_EXPORT_NAME: &str = "새 파일명";
pub const LIBRARY_EXPORT_INVALID_NAME: &str =
    "사용할 수 없는 파일명입니다. 예약 이름과 금지 문자를 피하고 원본 확장자를 유지해 주세요.";
pub const LIBRARY_EXPORT_FOLDER: &str = "내보낼 폴더";
pub const LIBRARY_EXPORT_SUBMIT: &str = "복사 시작";
pub const LIBRARY_EXPORT_CANCEL: &str = "닫기";
pub const LIBRARY_EXPORT_PREPARING: &str = "원본과 출력 위치를 확인하고 있습니다.";
pub const LIBRARY_EXPORT_COPYING: &str = "영상 복사 중";
pub const LIBRARY_EXPORT_VERIFY_STAGE: &str = "복사한 파일 확인 중";
pub const LIBRARY_EXPORT_VERIFY_SOURCE: &str = "원본 변경 여부 확인 중";
pub const LIBRARY_EXPORT_VERIFY_FINAL: &str = "새 파일 최종 확인 중";
pub const LIBRARY_EXPORT_DONE: &str = "복사 내보내기를 완료했습니다. 원본은 그대로 유지됩니다.";
pub const LIBRARY_EXPORT_STOP: &str = "복사 취소";
pub const LIBRARY_EXPORT_STOPPING: &str =
    "취소하고 있습니다. 파일 상태 확인이 끝날 때까지 기다려 주세요.";
pub const LIBRARY_EXPORT_RETAINED: &str = "파일 상태를 확정하지 못했거나 출력 파일이 남아 있습니다. 아래 경로를 확인해 주세요. 기존 파일과 원본은 삭제하지 않습니다.";
pub const LIBRARY_EXPORT_CANCELLED: &str = "복사를 취소했습니다. 원본은 그대로 유지됩니다.";
pub const LIBRARY_EXPORT_TIMEOUT: &str =
    "제한 시간 안에 복사를 완료하지 못했습니다. 원본은 그대로 유지됩니다.";
pub const LIBRARY_EXPORT_COLLISION: &str =
    "같은 이름의 파일이나 폴더가 있습니다. 다른 파일명 또는 폴더를 선택해 주세요.";
pub const LIBRARY_EXPORT_FAILED: &str =
    "복사를 완료하지 못했습니다. 선택 파일이나 저장 위치를 확인하고 다시 시도해 주세요.";

pub const TOOL_BACKGROUND_WORK: &str = "도구 작업 진행 중";

pub const SETTINGS_GENERAL: &str = "일반";

pub const SETTINGS_SAVE_LOCATION: &str = "저장 위치";

pub const SETTINGS_SAVE_LOCATION_HELP: &str = "다운로드 화면의 저장 위치를 바꾸면 이번 요청에만 사용합니다. 기본 폴더는 이 설정 화면에서 저장 위치 적용으로 기억합니다.";

pub const SETTINGS_DEFAULT_SAVE: &str = "기본 저장 폴더";
pub const SETTINGS_DEFAULT_SAVE_HELP: &str = "위치를 적용하면 다음 실행의 기본 저장 폴더와 파일 목록 위치로 기억합니다. 진행 중인 작업의 경로는 바꾸지 않습니다.";
pub const SETTINGS_DRAFT_SAVE: &str = "이번 다운로드의 저장 위치";

pub const MEDIA_DROP_FALLBACK: &str =
    "영상 파일을 놓으면 열립니다. 놓기 전 미리보기는 표시되지 않을 수 있습니다.";

// Root catalog owns integration; detail wording remains unchanged.
pub const LIBRARY_EXPORT_RETAINED_NOTICE: &str = "복사 내보내기 상태를 확정하지 못했습니다. 파일 목록에서 실패 정보를 확인하세요. 기존 파일과 원본은 삭제하지 않습니다.";

pub const HELP_BUTTON: &str = "?";
pub const DOWNLOAD_PROGRESS: &str = "현재 파일 전송";
pub const DOWNLOAD_PROGRESS_ESTIMATED: &str = "현재 파일 전송 (추정)";
pub const DOWNLOAD_PROGRESS_WAITING: &str = "진행량 확인 중";
pub const DOWNLOAD_PROGRESS_FINISHING: &str = "전송 완료 · 파일 처리 중";
pub const DOWNLOAD_PROGRESS_HELP: &str = "진행률은 현재 전송 중인 파일 기준입니다. 영상과 오디오를 따로 받으면 다음 파일에서 다시 시작할 수 있습니다. 전송이 끝나도 합치기와 최종 파일 확인이 남을 수 있습니다. 총 크기를 모르면 퍼센트를 표시하지 않습니다.";
pub const TOOL_STATUS_INSTALLED: &str = "설치됨";
pub const TOOL_STATUS_NOT_INSTALLED: &str = "설치 안 됨";
pub const TOOL_STATUS_CHECKING: &str = "확인 중";
pub const TOOL_STATUS_FAILED: &str = "확인 실패";
pub const TOOL_STATUS_RECORD_HELP: &str =
    "설치 기록 표시입니다. 사용·제거할 때 파일과 실행 버전을 다시 확인합니다.";
pub const TOOL_PINNED_ALREADY_INSTALLED: &str =
    "이 버전은 이미 설치되어 있습니다. 새 버전 확인이나 보관된 버전 사용을 선택하세요.";

pub const LIBRARY_SEARCH_LABEL: &str = "파일명·경로 검색";
pub const HISTORY_SEARCH_LABEL: &str = "작업 ID·URL 검색";
pub const LIST_SEARCH: &str = "검색";
pub const LIST_SEARCH_CLEAR: &str = "검색 지우기";
pub const LIST_SEARCH_HELP: &str =
    "현재 데이터베이스의 전체 기록에서 검색합니다. 다음 페이지로 결과를 더 확인할 수 있습니다.";
pub const LIST_SEARCH_LIMIT: &str = "검색어는 256바이트 이내로 입력하세요.";
pub const LIST_SEARCH_EMPTY: &str = "검색 결과가 없습니다.";
pub const LIST_SEARCH_STALE: &str = "새 검색을 확인할 때까지 이전 결과를 표시합니다.";

pub const LIST_SEARCH_APPLIED: &str = "표시 중인 검색어";
pub const LIST_SEARCH_ALL: &str = "전체 목록";

pub const TOOL_DETAILS_HELP_TITLE: &str = "도구와 설치 상태 안내";
pub const TOOL_CACHE_STATUS_SHORT: &str = "최근 확인 목록";

// Compact history status is separate from download completion/failure notifications.
pub const HISTORY_STATE_DOWNLOADED: &str = "완료";
pub const HISTORY_STATE_FAILED: &str = "실패";
pub const HISTORY_STATE_CANCELLED: &str = "취소됨";
pub const HISTORY_STATE_DOWNLOADING: &str = "다운로드 중";
pub const HISTORY_CANCELLED_NOTE: &str = "미완료 파일 보존 · 이력에서 재개 불가";
pub const HISTORY_DOWNLOADED_NOTE: &str = "최종 파일 확인 완료 · 재개 불가";
pub const HISTORY_FAILED_NOTE: &str = "실패 원인 보기에서 확인하세요.";
pub const HISTORY_OPTIONS_UNAVAILABLE_NOTE: &str = "원래 옵션을 확인할 수 없어 재개할 수 없습니다.";

// Root coordinates central catalog.rs integration; candidate module references these names.
pub const MEDIA_OPEN_CHOOSE: &str = "다른 앱 선택…";
pub const MEDIA_OPEN_BUSY: &str = "연결 앱을 여는 중…";
pub const MEDIA_OPEN_REMOTE: &str = "다운로드한 로컬 파일이 있어야 연결 앱으로 열 수 있습니다.";
pub const MEDIA_OPEN_NO_ASSOCIATION: &str =
    "연결 앱을 선택하지 못했습니다. ‘다른 앱 선택…’에서 다시 선택해 주세요.";

// Root integrates these history-independent tool footer labels into central catalog.
pub const TOOL_BACKGROUND_LOOKUP: &str = "설치된 도구 확인 중";
pub const TOOL_BACKGROUND_VERSION_CHECK: &str = "새 버전 확인 중";
pub const TOOL_BACKGROUND_INSTALL: &str = "도구 설치 중";
pub const TOOL_BACKGROUND_VERIFY: &str = "선택한 도구 검증 중";
pub const TOOL_BACKGROUND_REMOVE: &str = "도구 삭제 중";
pub const TOOL_BACKGROUND_SELECT: &str = "다운로드 도구 적용 중";

pub const MEDIA_OPEN_DEFAULT_HELP: &str =
    "기본 연결 앱으로 영상을 엽니다. 연결 앱이 없으면 앱 선택을 요청합니다.";
pub const MEDIA_OPEN_CHOOSE_HELP: &str = "이 영상을 열 다른 앱을 Windows에서 선택합니다.";
pub const MEDIA_OPEN_BLOCKED: &str = "현재 작업이 끝나거나 중단된 뒤 연결 앱으로 열 수 있습니다.";

pub const FEATURE_TOOL_REMOTE_REQUIRED: &str =
    "이 영상 확인에는 yt-dlp와 FFmpeg·ffprobe가 필요합니다.";
pub const FEATURE_TOOL_YTDLP_REQUIRED: &str = "이 기능에는 yt-dlp가 필요합니다.";
pub const FEATURE_TOOL_FFMPEG_REQUIRED: &str = "이 기능에는 FFmpeg·ffprobe가 필요합니다.";
pub const FEATURE_TOOL_DENO_PATH: &str =
    "선택한 Deno 경로를 확인하세요. Deno는 로컬 영상에 필요하지 않습니다.";
pub const FEATURE_TOOL_INVALID_PATH: &str =
    "지정한 경로가 올바르지 않습니다. 경로를 수정하거나 관리 도구를 선택하세요.";
pub const FEATURE_TOOL_CONFIGURE: &str = "도구 설정 확인…";
pub const FEATURE_TOOL_EXECUTION_FAILED: &str =
    "도구 실행에 실패했습니다. 설정과 실패 원인을 확인한 뒤 다시 시도하세요.";
pub const FEATURE_TOOL_DRAFT_PRESERVED: &str = "영상과 저장 구간은 그대로 유지됩니다.";
pub const FEATURE_TOOL_RETURN: &str = "영상 작업으로 돌아가기";
pub const FEATURE_TOOL_RETURN_RETRY: &str = "돌아가서 영상 다시 확인";
pub const FEATURE_TOOL_NEXT_YTDLP: &str = "yt-dlp 설정…";
pub const FEATURE_TOOL_NEXT_FFMPEG: &str = "FFmpeg 설정…";
pub const FEATURE_TOOL_NEXT_DENO: &str = "Deno 설정…";

pub const MEDIA_REVEAL_FILE: &str = "탐색기에서 파일 선택";
pub const MEDIA_REVEAL_HELP: &str =
    "저장된 폴더를 열고 이 파일을 선택합니다. 파일은 이동하거나 전송하지 않습니다.";
pub const STARTUP_CHECKING_TITLE: &str = "앱 준비 중";
pub const STARTUP_FAILED_TITLE: &str = "앱 준비를 완료하지 못했습니다";
pub const STARTUP_LOCKED_HELP: &str =
    "이전 작업 확인이 끝나면 다운로드 시작과 일부 설정을 사용할 수 있습니다.";

pub const APP_UPDATE_TITLE: &str = "앱 업데이트";
pub const APP_UPDATE_INSTALLED: &str = "현재 버전";
pub const APP_UPDATE_WINDOWS: &str = "Windows x64";
pub const APP_UPDATE_OTHER_PLATFORM: &str = "앱 업데이트는 Windows x64에서 먼저 지원합니다.";
pub const APP_UPDATE_CHANNEL: &str = "조회 채널";
pub const APP_UPDATE_STABLE: &str = "안정 버전";
pub const APP_UPDATE_PREVIEW: &str = "미리보기 버전";
pub const APP_UPDATE_CHECK: &str = "새 버전 확인";
pub const APP_UPDATE_CHECKING: &str = "새 버전 확인 중…";
pub const APP_UPDATE_CANCEL: &str = "조회 중단";
pub const APP_UPDATE_CANCELLING: &str = "조회를 중단하고 있습니다…";
pub const APP_UPDATE_CANCELLED: &str = "버전 확인을 중단했습니다.";
pub const APP_UPDATE_AVAILABLE: &str = "새 버전이 있습니다";
pub const APP_UPDATE_CURRENT: &str = "현재 버전보다 새로운 공개 버전이 없습니다.";
pub const APP_UPDATE_NONE: &str = "아직 공개된 버전이 없습니다.";
pub const APP_UPDATE_NONE_STABLE: &str =
    "아직 공개된 안정 버전이 없습니다. 미리보기 채널에서도 확인할 수 있습니다.";
pub const APP_UPDATE_NOTES: &str = "변경내역";
pub const APP_UPDATE_NOTES_EMPTY: &str = "이 릴리스에는 변경내역이 없습니다.";
pub const APP_UPDATE_NOTES_TRUNCATED: &str =
    "변경내역의 일부만 표시합니다. 전체 내용은 릴리스에서 확인하세요.";
pub const APP_UPDATE_SIGNED_METADATA_MISSING: &str =
    "이 버전은 앱 내 업데이트를 지원하지 않습니다. 릴리스에서 설치해 주세요.";
pub const APP_UPDATE_NONE_PREVIEW: &str = "최근 공개 릴리스에 미리보기 버전이 없습니다.";
pub const APP_UPDATE_OFFLINE: &str =
    "버전을 확인하지 못했습니다. 네트워크 연결을 확인하고 다시 시도하세요.";
pub const APP_UPDATE_TIMEOUT: &str = "응답 시간이 초과되었습니다. 잠시 후 다시 시도하세요.";
pub const APP_UPDATE_RATE_LIMIT: &str = "GitHub 조회 한도에 도달했습니다. 잠시 후 다시 시도하세요.";
pub const APP_UPDATE_INVALID: &str =
    "릴리스 정보를 확인하지 못했습니다. 릴리스 페이지에서 확인하세요.";
pub const APP_UPDATE_WORKER_FAILED: &str =
    "버전 확인을 시작하지 못했습니다. 앱을 다시 열고 시도하세요.";
pub const APP_UPDATE_RELEASE_OPEN: &str = "릴리스에서 설치";
pub const APP_UPDATE_RELEASE_VIEW: &str = "릴리스 보기";
pub const APP_UPDATE_ALL_RELEASES: &str = "모든 릴리스 보기";
pub const APP_UPDATE_MANUAL_ONLY: &str =
    "앱 내 자동 설치는 아직 제공하지 않습니다. 새 버전은 릴리스에서 설치하세요.";
pub const APP_UPDATE_CACHED: &str = "이전 확인 결과";
pub const APP_UPDATE_INITIAL: &str = "확인을 누르면 GitHub의 공개 릴리스에서 새 버전을 조회합니다.";
pub const APP_UPDATE_VERSION_HELP: &str = "앱 버전과 개발 검수 번호(v37 등)는 다릅니다. 다운로드 도구 업데이트는 도구 설정에서 관리합니다.";

pub const APP_UPDATE_PREPARE: &str = "다운로드 및 준비";

pub const APP_UPDATE_RESTART: &str = "다시 시작하여 적용";

pub const APP_UPDATE_PREPARED: &str = "새 버전이 준비되었습니다. 다시 시작하면 적용합니다.";

pub const APP_UPDATE_PREPARING: &str = "새 버전을 준비하고 있습니다…";

pub const APP_UPDATE_VERIFYING: &str = "업데이트 서명을 확인하고 있습니다…";

pub const APP_UPDATE_DOWNLOADING: &str = "업데이트를 다운로드하고 있습니다…";

pub const APP_UPDATE_STAGING: &str = "다운로드한 패키지를 검사하고 있습니다…";

pub const APP_UPDATE_STOP: &str = "준비 중단";

pub const APP_UPDATE_STOPPING: &str = "준비를 중단하고 있습니다…";

pub const APP_UPDATE_RESTARTING: &str = "재시작 승인을 기다리고 있습니다…";

pub const APP_UPDATE_STARTUP_WAIT: &str = "시작 복구를 마친 후 업데이트할 수 있습니다.";

pub const APP_UPDATE_TASK_WAIT: &str = "진행 중인 작업을 마친 후 업데이트할 수 있습니다.";

pub const APP_UPDATE_SUPPORT_MANUAL: &str =
    "이 실행 방식에서는 자동 업데이트를 지원하지 않습니다. 릴리스에서 설치해 주세요.";

pub const APP_UPDATE_TRUST_UNAVAILABLE: &str =
    "업데이트 서명키가 설정되지 않았습니다. 릴리스에서 설치해 주세요.";

pub const APP_UPDATE_UPDATE_CANCELLED: &str =
    "업데이트 준비를 중단했습니다. 현재 앱은 그대로 유지됩니다.";

pub const APP_UPDATE_VERIFY_FAILED: &str =
    "업데이트 검증에 실패했습니다. 현재 앱은 그대로 유지됩니다.";

pub const APP_UPDATE_DISK_FAILED: &str =
    "업데이트를 저장하지 못했습니다. 여유 공간과 접근 권한을 확인하세요.";

pub const APP_UPDATE_PREPARE_FAILED: &str =
    "새 버전을 준비하지 못했습니다. 현재 앱은 그대로 유지됩니다.";

pub const APP_UPDATE_EXPIRED: &str = "업데이트 정보가 만료되었습니다. 새 버전을 다시 확인하세요.";

pub const APP_UPDATE_RESTART_REFUSED: &str =
    "현재 상태에서는 다시 시작할 수 없습니다. 준비된 버전은 유지됩니다.";

pub const APP_UPDATE_PROTOCOL_FAILED: &str =
    "업데이트 연결이 끊겼습니다. 앱을 다시 실행한 후 확인하세요.";

pub const APP_UPDATE_PROFILE_FAILED: &str =
    "앱 데이터 호환성을 확인하지 못해 업데이트를 적용하지 않았습니다.";

pub const APP_UPDATE_UPDATE_WORKER_FAILED: &str =
    "업데이트 작업을 완료하지 못했습니다. 현재 앱은 그대로 유지됩니다.";

pub const APP_UPDATE_RECOVERED: &str =
    "새 버전을 실행하지 못해 이전 버전으로 돌아왔습니다. 업데이트를 다시 확인해 주세요.";

pub const APP_UPDATE_RECOVERY_CONFIRM: &str = "확인";
