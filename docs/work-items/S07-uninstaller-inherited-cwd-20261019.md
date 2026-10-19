# S07: 제거기의 작업 폴더 상속으로 인한 등록 정리 실패

- 상태: completed
- 분류: bug
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: Root — crates/app/src/installer_uninstall.rs, Cargo.toml/Cargo.lock 버전, CONTEXT/ADR/배포문서; QA agent — ignored private-qa observer/probe만
- 선행 작업: v0.1.3 source0828 CI와 실제 private 제거 실패

## 사용자 결과

Windows 앱 제거가 본체 제거 뒤 등록 정리까지 완료되도록 한다. 제거 진입의 부모 작업 폴더를 임시 제거 자식이 상속해 삭제할 지원 폴더를 잠그는 가설을 검증한다.

## 계약과 범위

기존 v0.1.3 태그는 변경하지 않고 공개하지 않는다. 새 수정본은 v0.1.4 후보로 검증한다. current-user/private desktop/원 cap·소유 Job·취소·보존 계약을 유지한다. 전역 CWD 변경이나 공유 모드 완화는 하지 않는다. 기존 사용자 설치/등록 및 실패 증거를 임의 정리하지 않는다. 새 의존성/상주 비용 추가0.

## 인수 조건

원 trigger인 지원 폴더 CWD에서 실행한 제거 진입으로 설치→취소→제거→동일 위치 재설치→제거를 검증한다. 작업 폴더 상속/중립 임시 폴더를 사용하는 실제 child의 원 directory DELETE reopen 결과를 대조한다. 영상·프로필·도구·외부 journal 보존과 앱 목록/shortcut/support 부재를 함께 확인하며 exit0만으로 성공을 판단하지 않는다. 수정된 source의 fmt/clippy/workspace test/release build, 새 태그 CI와 동일 run ZIP/공개 서명·asset 검증을 완료한다.

## 검증과 handoff

v15 actual QA에서 설치·등록·COM shortcut·실행·취소·원 데이터 보존은 통과했다. remove UI는 일부 제거 실패를 표시했고 본체는 부재, own registry/shortcut/support 두 파일은 모두 남았다. 원 JSON/log/NativeID/PNG는 ignored private-qa에 보존한다. 이 결과를 성공 제거로 보고하지 않는다. 가설은 CWD상속, source helper HANDLE 잔존, 등록 관측 변경 순이다. 최소 native probe와 원 UI loop로 구분한다.

진단 결과: 별도 native child CWD 대조에서 support CWD 생존 중 named DELETE open이 OS32로 거절됐고 종료 후 성공했다. 중립 임시 CWD는 DOS/extended 두 표기 모두 자식 생존 중 원 support nativeID DELETE open이 성공했다. 원 자식들은 모두 자연 exit0/Job0, 삭제 실행0, 기존 등록 변경0이다. Root가 원 JSON을 독립 검토했다.

최소 수정은 handoff CreateProcessW의 lpCurrentDirectory를 NULL 대신 검증된 private copy 부모 경로로 명시한다. 전역 CWD나 공유 모드는 바꾸지 않는다. v0.1.4의 전체 native 검사 중이며 성공 제거를 아직 주장하지 않는다. 기존 actual UI loop가 정확한 호출 경계를 재현하므로 shallow 경로 반환 단위 테스트를 대신 추가하지 않고 같은 support-CWD trigger로 회귀한다.

## v0.1.4 실제 완료 근거

[공개 v0.1.4](https://github.com/joongi007/dlpad/releases/tag/v0.1.4)의 소스는 `e8d1007a792c17319e5498cfc6182d174745a4ab`, [태그 CI](https://github.com/joongi007/dlpad/actions/runs/39157614958)와 [main CI](https://github.com/joongi007/dlpad/actions/runs/39157611977)는 모두 성공이다. `python3 .local-tools/windows-shell-20261019/run-native-cwd-fix.py`에서 fmt·strict workspace/all-target clippy·등록12회귀·workspace1,856 PASS/0 FAIL/44 ignored·release build가 모두0이다. hosted 원 stdout도같은1,856/0/44이며 원7개 updater 회귀의 자연 Job0/로그 hash도대조했다.

원 CI ZIP으로 private 실제 설치→등록→앱실행→취소→제거→같은 위치 재설치→재제거를 확인했다. 다섯 단계 원 parent/UI/Job은자연0이며 source/privateDesktop/inputDesktop불변을확인했다. 두 외부 journal의 원5기록씩 SHA를최종제거후다시읽어대조했다. Root가 원 로그와 주요 PNG4개를직접검수했다. 기존 공개0.1.2에 새 설치기로 명시 등록해 원bundle/journal nativeID/bytes 보존, 기존CPU실행·자연Job0, 실제화면3개도확인했다. 합성 profile/media/tools만사용했고 실제사용자 Documents/영상은사용하지않았다.

설치 ZIP SHA256 `7c21482631e666486fe0a190d358e3413f158855441b471aee416f5969502576`, installer SHA256 `bbed56461a9cda0d12928077e491fb81c2805170462bee029fa6de6580940273`이다. 검수한 같은원본을 공개했고7개asset을인증없이내려받아각bytes/SHA를대조했다. 로컬DPAPI개인키는전송하지않고compiled공개키로검증한공개metadata/signature만공급했다. Root원인수는 ignored publication-v0.1.4/acceptance.json 및 legacy-acceptance.json, 공개전달검사는 public-download-review.json에기록했다.

제한: Windows API/COM/private HWND 검수이며 물리설정/제어판·새제거기200%·스크린리더·UNC·다른OS검증은아니다. cold부분제거복구·baseline버전동기화·임시제거복사본자동정리는후속이다. 실제공개1.4앱GUI업데이트전환을별도실행했다고주장하지않으며동일프로토콜signed합성7회귀/공개metadata원bytes검증과구분한다. 기존0.1.2사용자의OS등록은새설치기에서명시해야한다.
