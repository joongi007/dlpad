# S07: 현재 사용자 Windows 앱 등록·시작 메뉴

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R10, R11
- 담당/수정 범위: engine agent — 독립 os-registration worktree의 crates/engine/src/app_install/os_registration.rs 및 전용 tests 파일; Root가 app_install.rs exports/Cargo/shared계약 통합
- 선행 작업: v0.1.2 공개와 사용자 Windows등록/제거 개발·배포 명시승인

## 사용자 결과

현재 사용자 Windows 앱 등록·시작 메뉴를 구현해 일반 Windows 앱처럼 설치 목록에서 찾고 실행·제거한다.

## 계약과 범위

Rust native/current-user, 관리자권한·WebView·shell문자열없음. 기존ZIP7/manifest5와v0.1.2 updater읽기호환유지. 앱소유 shell support는bundle외부 별도namespace이며기존미디어·프로필·외부도구·미확인항목자동삭제없음. sharedexports/Cargo/lock/버전은Root만변경한다. agent는자기worktree/명시파일만편집하며최대3개, Root가원결과를독립검증한다.

## 인수 조건

실제HKCU Uninstall항목·version/icon/정확UninstallString·시작메뉴 .lnk 생성/읽기/제거, collision/변조/취소/권한/실행중 실패, 외부항목보존. 테스트는임시root/별도testregistry/별도StartMenu경로만사용.

## 검증과 handoff

현재환경Windows우선/WSL개발, D:338MiB·/tmp586GiB를확인했다. 신규전체cache다운로드대신기존C:ownedtarget/tmp를재사용한다. 원root .git/index.lock은보존하고정상ec77f46 worktree에서분리한다. 실제명령/결과·비용·미검증항목을완료시기록한다.
