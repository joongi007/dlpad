# S00: CPU renderer 중간 캔버스 비용 대조

- 상태: done (현재CPU 후보의좁은renderer설정선택)
- 분류: performance
- 우선순위: P0
- 요구사항: R01, R03, R11
- 담당/수정 범위: root — crates/app/src/software_host.rs, 본 문서, 전용 evidence
- 선행 작업: metrics mailbox 및 DB byte guards 전체 Windows 회귀; 고정 GUI 비용 수집기

## 사용자 결과

목록10k 첫 실행에서80MiB를 넘는 메모리의 감소 가능성과 CPU·화면 영향를 실측한다.

## 계약과 범위

현재 egui_software_backend0.0.2는 default caching=true로 intermediate canvas를 보관한다.960x640 RGBA 자체는2,457,600bytes이며 실제 전체캐시메모리와 같다고 주장하지 않는다. 기존 with_caching(false) 대조는 추가의존성/lockfile 변경 없이 가능하지만 upstream은 더 느릴 수 있다고 명시한다. 현재상태3fresh 대조를 먼저측정하고 단일설정만 바꿔3fresh 비용·화면을 확인한다. 기준80MiB/1%/10초/warmup2/private desktop/자연exit0·Job0/input불변을유지한다. 예산변경·WorkingSet trim·visible fallback 없다. 다른native시험과비용측정을겹치지않는다.

## 인수 조건

same source except renderer setting 대조와 실제PNG·스크롤/hidden texture 기존회귀, 전체workspace검증. memory만통과하고CPU나시각동작이실패하면채택하지않는다. cold/quietHost/장시간성능과최종스택인수는별도이다.

## 검증과 handoff

baseline이후소스수정한다. upstream설정지원은로컬고정registry코드읽기로확인했다. 현재메모리문제원인확정이나cacheoff채택완료가아니다.

root는singlechangedfile software_host.rs를 baseline vs direct 전체SHA로확인했다. cached baseline 첫WS84480000/CPU0.77413% memoryFAIL, 다른WS79757312/79400960·CPU0.46467/0.46479% PASS이다. direct3회는WS79237120/74010624/73965568·CPU0.30953/0.31239/0.30923%로전부PASS. 실제cached/direct첫idlePNG의동일한글목록·controls를직접비교했고 각rootfixture10k3byte/read-only DB10k 및자연exit0/Job0/desktop닫힘/input불변을독립검증했다. fmt/clippy/test/release와actualdirect hidden texture partial/free회귀도PASS했다. 추가버전/의존성/lock변경0. 현재optional CPU후보에with_caching(false)를채택한다. 관측된memory감소전체를canvas바이트에단독귀속하거나3×10sec를8h/quietHost/IME/최종stack인수로확대하지않는다. 근거 windows-renderer-direct-cost-20261005-resume/root-review.json; 기존실패모두보존.
