# S00: 작업 이력의 짧은 상태와 본문 재개 제한

- 상태: prepared (scratch 구현·회귀 작성, 통합 검증 전)
- 분류: UX-fix
- 우선순위: P0
- 요구사항: R07/R08/R13, S00 실제 상태/레이아웃 인수
- 담당/수정 범위: p0_audit — B/history-status-compact-v1 후보와 초안만. Root — catalog/accesssible_history 통합·Cargo/native/최종 검증
- 선행 작업: 기존 v28 history ID/URL private 결과 Root 픽셀 관측, 현재 package/help source 동결

## 사용자 결과

작업 이력에서 상태가 긴 다운로드 알림 문장으로 표시되는 대신 완료/실패/취소됨/다운로드 중으로 표시된다. 미완료 보존·취소/완료 재개 불가·원래 옵션 확인 불가의 이유와 실패 상세 안내는 본문에서 읽을 수 있다.

## 계약과 범위

공용 다운로드 catalog 원문은 유지하고 history 전용 문자열8개만 추가한다. metadata/Resume action/32px geometry/entity ID/eligibility/worker/schema는 변경하지 않는다. Root 공유 Search::reveal_result의 유효 one-shot 소비 bool 반환에 의존한다. 원IDLabel response ID/rect는 유지하며 전체 metadata+중요안내 group이실제clip에fit할때만별도scroll_to_rect를요청한다. oversized면원ID우선이다. 키보드 focus/label capability/queue lifetime·Stop 규칙은 유지한다.

## 인수 조건

실제 Context 100/200%·125logical body의 긴ID와버전·4상태와125/60logical body의16상황에서 compact 상태와 제한 Label을 fully paint하고 focus를 보존한다. disabled Resume Click은 command를 만들지 않는다. 기존 entity/focus/reorder/removed row/32px/pending/readiness 회귀가 유지된다. Root가 source/current PE native history ID/URL screen을 직접 검수한다. 단위검사를 전체GUI 인수로 대체하지 않는다.

## 검증과 handoff

실제 작성된 RED/GREEN 파일과 test가 있고 scratch rustfmt exit0. Cargo/native/actual RED·GREEN 실행은0이며 계획된 결과를 PASS로 쓰지 않는다. Live/S/shared catalog 원문 변경0, source2파일 base SHA 불변 확인. Root 공유list_search bool계약/의존채택·stage/새 전체검사/새 PE 비용 및 실제 시각 인수는 후속이다.

Root 공유파일 list_search.rs는 별도 history-reveal-contract-v1의 bool 반환 계약으로 조정합니다. Live는28로동결하며 S에서만 실제 RED/GREEN을 검증합니다.

Root actual RED compile 성공, compact Cancelled status 실제paint 조건에서0 PASS/1 FAIL로재현했습니다. Live및S스냅샷불변/원Job자연종료·private/input불변확인. 다음GREEN은S에서만실행하며제공설치본28은그대로입니다.

Root GREEN-v1 실제 결과는 기존 History 11 PASS/새 시험 1 FAIL이다. Enter 후 pending 비활성 입력에 enabled-only Focus를 보내는 시험 전제에서 실패했으며 제품 gate는 변경하지 않았다. v2는 Enter 직후 실제 focus/None을 기록하고 기존 fixture에 cfg(test) disabled node Click 실패경로 helper만 더한다. body125/60·zoom100/200·4상태의 paint와 명령 미발생 조건은 유지한다. Root가 S 전용으로 3파일 stage했고 GREEN-v2 실제 실행 중이며 원 실패 자료는 보존한다.

Root GREEN-v2 실제 Context 결과는 12 PASS/0 FAIL이며 source 및 S 후보가 검사 동안 불변이다. strict workspace/all-targets clippy도 실제0으로 통과했다. 현재 제품 Live/설치본은28이며 원 공개 구간 저장의 읽기 검증과 공유 도움말 후보 검토 뒤 채택한다. 전체 새 검사 및 native 픽셀 검수 전에는 제품 완료로 표시하지 않는다.


Root가 Help 보호배치와 함께 관련3파일을 Live에 채택했다. 원 Label rect/ID와 실행 gate를 유지했고 새 whole29 검사 진행 중이다. 최신 사용자 ZIP은28이며 새 GUI 검수 전 제품 UI 인수 완료가 아니다.

Root 새 whole29 완료: fmt/strict workspace all-targets clippy/test/release 실제0, 1,613 PASS/0 FAIL/42 ignored·43harness·source674이며 원로그/current3PE/lock을 독립 대조했다. CPU10,321,920B(+6,144B), CLI978,432B, installer4,432,896B다. native 화면/최신 ZIP은 후속 진행이며 전체P0로 확대하지 않는다.
