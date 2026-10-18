# S07: Windows 제거 진입과 확인 화면

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R10, R11
- 담당/수정 범위: app agent — 독립 app-uninstall worktree의 crates/app/src/installer_uninstall.rs, installer_uninstall_catalog.rs만; Root가 installer_main entry/배포graph/공유API 통합
- 선행 작업: v0.1.2 공개와 사용자 Windows등록/제거 개발·배포 명시승인

## 사용자 결과

Windows 제거 진입과 확인 화면를 구현해 일반 Windows 앱처럼 설치 목록에서 찾고 실행·제거한다.

## 계약과 범위

Rust native/current-user, 관리자권한·WebView·shell문자열없음. 기존ZIP7/manifest5와v0.1.2 updater읽기호환유지. 앱소유 shell support는bundle외부 별도namespace이며기존미디어·프로필·외부도구·미확인항목자동삭제없음. sharedexports/Cargo/lock/버전은Root만변경한다. agent는자기worktree/명시파일만편집하며최대3개, Root가원결과를독립검증한다.

## 인수 조건

설치원본ZIP없이설치된앱목록에서제거화면진입, 영상/설정/도구보존안내·명시확인·취소·worker실행·실패표시. selfimage제거는검증된temporary handoff계약. privateDesktop실제GUI검수.

## 검증과 handoff

현재환경Windows우선/WSL개발, D:338MiB·/tmp586GiB를확인했다. 신규전체cache다운로드대신기존C:ownedtarget/tmp를재사용한다. 원root .git/index.lock은보존하고정상ec77f46 worktree에서분리한다. 실제명령/결과·비용·미검증항목을완료시기록한다.
