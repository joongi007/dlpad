# S04: 앱 관리 도구 검사·자동 선택 UX

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R02, R06, R08
- 담당/수정 범위: root — accessible.rs/catalog.rs·공유 계약·통합; windows_auto — accessible_tools.rs; advanced_units — accessible_tool_preferences.rs
- 선행 작업: Windows 관리 설치·선택·목록 identity 및 기존 strict tool verifier

## 사용자 결과

도구별 호환·무결성·실제 버전 검증 상태로 자동 사용과 선택 가능 여부를 결정한다. 없음/잘못됨은 이유를 표시하고, 명시 외부 경로·환경값·수동/늦은 입력을 보존한다. 검사 자체는 활성 버전·파일을 바꾸거나 다운로드하지 않는다. 설치 성공 뒤 자동 모드에만 사용을 연결한다.

## 계약과 범위

공유 schema/Cargo·계약은 root가 조정한다. 동시에 같은 파일을 수정하지 않고 후보는 프로젝트 소유 영구 recovery 디렉터리에 보존한다. 기존 /tmp 준비물은 WSL 재시작 때 소실됐으며 옛 proof를 복원했다고 표현하지 않는다.

## 인수 조건

원 검증·취소·CloseWait·현재 다운로드 pin·부분 실패 보존 및 실제 입력/worker 실패 경로를 검증한다. UI thread I/O·새 schema/Cargo 변경은 금지한다.

## 검증과 handoff

현재 후보 구현·API 합의 중이다. 아직 제품 통합/실제 PASS가 아니다. v19 IME 전체 검증 재개와 병행하되 source freeze 동안 새 tool 후보를 live에 채택하지 않는다.

## 실제 격리 회귀 검증

`managed-tool-isolated-prefs-red-20261013`은 원래 제품 생산 코드를 유지한 actual Context 검사에서 검증되지 않은 자동 선택이 활성화됨을 0 PASS/1 FAIL로 재현했다. 수정 후보 `managed-tool-isolated-prefs-green-20261013`의 네 실제 Windows Context/저장소 회귀는 4 PASS/0 FAIL이다. root가 원시 host 종료·source/candidate 보존·각 actual1 결과를 독립 검토했다. 후보는 아직 live 채택되지 않았으며 전체 UI/worker 인수는 후속이다.

## Worker 및 통합 후보

`managed-tool-isolated-worker-green-20261013`의 actual 5 filters는 3+1+1+4+1=10 PASS/0 FAIL이다. 현재 Main 연결 전 사용되지 않는 Checking/take_inspection 경고는 보존하며 strict clippy 완료로 보고하지 않는다. root 후보 `root/main-v20`, `root/preferences-v3` 및 담당 worker v4는 요청 시 슬롯별 path/mode revision ticket을 전달해 이전 명시적 Auto 전환을 허용하면서 늦은 Manual·직접 편집·환경 변수(빈 값 포함)·CloseWait를 보호한다. 저장소 로드 실패는 Automatic 복구 권한을 부여하지 않으며 기존 preferences 오류 표시가 담당한다. 후보는 아직 live 미채택이고 실제 통합 회귀를 이어서 수행한다.

## 전체 회귀에서 발견한 세 기존 동작의 수정

격리 CPU 앱 전체 최초 결과는 363 PASS/3 FAIL/4 ignored였다. 기존 빈 Use 두 곳은 기존 TOOL_EMPTY 안내를 복원했다. 나머지 첫 목록 조회는 파일 잠금·해시 이후 native 버전 실행 구간에서 원래 4초 한도를 넘김을 실제 계측했다(`refresh-core-diagnostic-v5b`). 명백한 비실행 파일은 Windows DOS prefix만 먼저 거부한다. 이는 full PE/publisher 검증 완료가 아니다. `refresh-green-v6b` 원래 3회귀는 3 PASS/0 FAIL이며 첫 페이지 실제 test duration은 0.11초, 기존 4초 guard와 inventory-only 합성 fixture를 유지했다. yt-dlp Install도 inactive API로 연결하여 Manual dispatch가 먼저 active를 바꾸지 않도록 수정했다. 임시 진단 로그는 채택 후보에서 제거했다.

## 제품 소스 채택 checkpoint

root가 8개 격리 검증 파일을 제품 소스에 채택했다. 원 v19 8파일 backup·각 original/accepted hash·중간 intent/receipt는 `.local-tools/session-recovery-20261013/live-adoption-v20/`에 보존했다. Catalog fmt만 후속 적용했다. `windows-msvc-managed-tool-auto-whole-20261013`에서 새 제품 fmt/clippy/test/release를 실제 실행 중이다. 완료 전에는 새 전체 PASS·설치본 갱신·전체 UI 또는 최신 업데이트 완료로 보고하지 않는다.

## 2026-10-13 v20 통합 검증 완료 범위

Windows MSVC `windows-msvc-managed-tool-auto-whole-20261013`의 fmt/clippy/workspace test/release build는 모두 통과했다(1357 PASS/0 FAIL/37 ignored). root가 whole 원본 로그·657 source map·현재 실행파일 3개의 정확한 SHA/bytes를 독립 확인했다. v20 앱 63 PNG는 root11+하위52 분담 직접 검수 및 현재 image pin 일치를 확인했고, installer 주요8 PNG와 정상/음수 launch raw lifetime/source/input pin도 확인했다. 새 다운로드 환경 설명 배치는 별도 probe geometry 오차가 있어 미인수다. P1 최신 조회/동적 공급/UI 통합은 후보이며 이 설치본에 포함되지 않는다. 전체 P0·물리 IME·정식 서명/공개 배포 완료를 뜻하지 않는다.
