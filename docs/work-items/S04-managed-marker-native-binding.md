# S04: 관리 실행 기록의 원 native identity

- 상태: in-progress
- 분류: safety
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: root — crates/engine/src/tools/execution_marker.rs, 이 문서
- 선행 작업: managed pending failclosed 및 Windows original handle 시험

## 사용자 결과

새 pending 기록은 생성 당시 retained root/marker HANDLE의 native IDs와 session nonce를 version2 JSON으로 보존한다. 기존 unknown/v1 pending은 계속 관리 변경을 차단한다.

## 계약과 범위

root가 공유 기록 schema를 조정한다. bounded4096bytes/denyunknownfields, 실제 create-only File에서 IDs를 관측한다. 새 의존성 없이 기존serde/nativeidentity를 재사용하며 startup 자동삭제 API는 추가하지 않는다. 원파일ID는 종료receipt/ACL 검증을 대체하지 않는다. readerjournal/lock binding과 모든 실행의 실제 terminal receipt는 후속이다.

## 인수 조건

실제원marker/root IDs 기록일치, nonce와 version2/크기한도, 정상finish/Drop보존/변조 및 namespace 교체 거절 회귀를 실행한다. Windows 네이티브 scoped 재검증이 필요하며 Linux 통과로 대체하지 않는다.

## 검증과 handoff

구현·실행 결과는 아래 추가한다. 자동회수 완료로 주장하지 않는다.

### 실제 scoped 결과

cargo fmt 완료, engine tools::execution 회귀9개PASS(원nativebinding/unknownfield거절 포함). 신규Windows전체collector managed-marker-v2-windows-workspace 실행중이며아직PASS주장하지않는다. version2는원root/marker ID만기록하며lock/journal/receipt/ACL연결및automaticrecovery는미구현이다. 기존v1/unknownpending은계속보존한다.
