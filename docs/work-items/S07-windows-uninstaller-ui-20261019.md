# S07: Windows 제거 진입과 확인 화면

- 상태: completed
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

## v0.1.4 실제 완료 근거

[공개 v0.1.4](https://github.com/joongi007/dlpad/releases/tag/v0.1.4)의 소스는 `e8d1007a792c17319e5498cfc6182d174745a4ab`, [태그 CI](https://github.com/joongi007/dlpad/actions/runs/39157614958)와 [main CI](https://github.com/joongi007/dlpad/actions/runs/39157611977)는 모두 성공이다. `python3 .local-tools/windows-shell-20261019/run-native-cwd-fix.py`에서 fmt·strict workspace/all-target clippy·등록12회귀·workspace1,856 PASS/0 FAIL/44 ignored·release build가 모두0이다. hosted 원 stdout도같은1,856/0/44이며 원7개 updater 회귀의 자연 Job0/로그 hash도대조했다.

원 CI ZIP으로 private 실제 설치→등록→앱실행→취소→제거→같은 위치 재설치→재제거를 확인했다. 다섯 단계 원 parent/UI/Job은자연0이며 source/privateDesktop/inputDesktop불변을확인했다. 두 외부 journal의 원5기록씩 SHA를최종제거후다시읽어대조했다. Root가 원 로그와 주요 PNG4개를직접검수했다. 기존 공개0.1.2에 새 설치기로 명시 등록해 원bundle/journal nativeID/bytes 보존, 기존CPU실행·자연Job0, 실제화면3개도확인했다. 합성 profile/media/tools만사용했고 실제사용자 Documents/영상은사용하지않았다.

설치 ZIP SHA256 `7c21482631e666486fe0a190d358e3413f158855441b471aee416f5969502576`, installer SHA256 `bbed56461a9cda0d12928077e491fb81c2805170462bee029fa6de6580940273`이다. 검수한 같은원본을 공개했고7개asset을인증없이내려받아각bytes/SHA를대조했다. 로컬DPAPI개인키는전송하지않고compiled공개키로검증한공개metadata/signature만공급했다. Root원인수는 ignored publication-v0.1.4/acceptance.json 및 legacy-acceptance.json, 공개전달검사는 public-download-review.json에기록했다.

제한: Windows API/COM/private HWND 검수이며 물리설정/제어판·새제거기200%·스크린리더·UNC·다른OS검증은아니다. cold부분제거복구·baseline버전동기화·임시제거복사본자동정리는후속이다. 실제공개1.4앱GUI업데이트전환을별도실행했다고주장하지않으며동일프로토콜signed합성7회귀/공개metadata원bytes검증과구분한다. 기존0.1.2사용자의OS등록은새설치기에서명시해야한다.
