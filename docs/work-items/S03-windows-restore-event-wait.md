# S03: Windows 복귀 pipe의 idle event 대기

- 상태: proposed
- 분류: performance
- 우선순위: P0
- 요구사항: R08
- 담당/수정 범위: root crates/engine/src/activation/windows.rs·공통 문서·native 검증; diagnostic_cache /tmp 독립 race 검토
- 선행 작업: 현재2ms전체/GUI계측 원결과; 제품source freeze 종료

## 사용자 결과

복귀 요청이 없을 때 원 named pipe의 완료 이벤트를 기다리고 반복20ms 확인의 idle 비용을 줄인다. 숨김·재실행·정상종료 동작은 유지한다.

## 계약과 범위

original overlapped/event capability·nonce·SID·receipt·250ms IO기한·CancelIoEx 후drain을유지한다. serverDrop의원stop+CancelIoEx가pending완료event를깨우는지race를확인하기전제품수정하지않는다. 추가thread/dep/schema/사용자desktop변경0. 순수stopflag만있는다른caller의대기를무조건무한대기로바꾸지않는다.

## 인수 조건

실제Windows idleConnect의드랍/직전취소/클라이언트요청·transfer250ms timeout/부분receipt·원별foreign보존/소유helper종료를회귀한다. 원pending돌입관측과종료≤기존기한을구분한다. actualthreadCPU/샘플해상도·변경효과를기록하며전체GUI기준은별도로유지한다.

## 검증과 handoff

현재read-only에서Pending.finish가20ms event wait를반복하고serverDrop은원CancelIoEx를호출함을확인했다. APIrace/기한검토초안만착수하며제품미수정·비용개선미검증이다.
