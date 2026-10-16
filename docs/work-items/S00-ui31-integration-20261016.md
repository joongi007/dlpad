# S00: 영상 상세 열기·도구 진행 표시·작은 도움말 통합

- 상태: in_progress / 격리 검사 통과, 소스 채택·전체 검사 준비
- 분류: integration
- 우선순위: P0
- 요구사항: R03/R05/R07/R08/R10/R11/R13
- 담당/수정 범위: Root만 crates/app/src/{accessible.rs,accessible_cpu_main.rs,accessible_main.rs,catalog.rs,external_open_broker.rs,library_open.rs,media_workspace.rs,media_detail_open.rs,accessible_tools.rs,ui_components.rs}, crates/engine/src/{media_preview.rs,tools/staging_recovery.rs}
- 선행 작업: Windows S 선택20PASS·sparse2PASS·Help실제RED0/1→GREEN1/0·기존Help17PASS·strictworkspace/all-targetsclippy0

## 사용자 결과

영상 상세에서 기본 앱·다른 앱 선택을 요청하고, 도구 조회·버전 확인·설치·검증·삭제·적용 진행을 구분하며, 작은 창의 긴 도움말을 보호 입력 바깥 공간에서 스크롤한다.

## 계약과 범위

원 Prepared local pin과 기존 외부 앱 검증/worker/Job 경계를 유지한다. remote URL은 로컬 파일로 열지 않는다. chooser는 사용자 소유 Explorer에 위임하고 앱-owned modal을 만들지 않는다. 원 cache·busy·취소·종료 gate를 유지한다. sparse는Windows 합성512MiB 논리경계시험에만적용한다. Cargo/schema/lock변경없다. 아직재현하지못한History후보는채택하지않는다.

## 인수 조건

전체Windows fmt/clippy/test/release·원로그/675같은예상값없는실제sourcecount/3PE 대조 후 실제현재PE native를검수한다. private desktop만사용하며viewer/chooser의사용자desktop실행은자동화하지않는다. UIA가제공하지않는popupframe/scrollbounds를발명하지않고원PNG/관측한계를기록한다. 새패키지는실제검증후에만사용자안내를바꾼다.

## 검증과 handoff

Root통합helper v1의relative경로가read-only대소문자alias로해석되어첫backupmkdir전실패했다. Live변경0이며v2는명시lowercaseRoot를사용한다. 격리strictclippy2경고수정후실제0을확인했다. 전체검사와최신native/패키지는아직미실행이다. v30검수ZIP은보존한다.

실제Windowswhole31 종료0·Root원로그/3PE/source675/lock대조완료:1,636PASS/0FAIL/42ignored/43harness. CPU10,351,104B(v30대비+28,160B),CLI/installer크기는같으나SHA는다르므로각새pin을사용한다. native첫시도는media helper Invoke오류,Help helperC#타입오류로실패해새helper를진단중이다. 제품실패/전체UI통과로미리단정하지않는다.
