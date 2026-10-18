# S07: 제거기의 작업 폴더 상속으로 인한 등록 정리 실패

- 상태: in-progress
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
