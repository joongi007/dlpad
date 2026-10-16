# S01: 목록 재확인의 DB 열기 비용 감사

- 상태: done
- 분류: performance
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: metrics_recovery /tmp 실험초안·읽기검토; root 프로젝트 게시·실행·통합검증·본 문서
- 선행 작업: 동일목록 UI 전달 구현/전체검증, 현재 GUI 측정 종료

## 사용자 결과

GUI 목록 재확인의 실제 비용과 재사용 DB 예제의 비용 차이를 확인하여 후속 변경의 근거를 만든다.

## 계약과 범위

현재 app은 scan_inner에서 Store를 새로 연다. 캐싱으로 정책을 바꾸기 전에 temp 합성1만파일로 DB 열기와 공개scan 비용을 분리 측정한다. 초안은 /tmp만 작성하며 sourcefreeze 동안 native 실험을 실행하지 않는다. 외부 DB/root namespace 교체에서 새검증을 생략하는 캐싱을 자동 채택하지 않는다. Cargo/schema 변경은 root만 조정한다.

## 인수 조건

열기/scan 각각 실제 native CPU·wall, 원 fixture/sentinel·DB integrity/10000행·reopen 보존 확인, 기존 예제와 호출 차이 기록. GUI 부하 원인으로 단정하지 않는다.

## 검증과 handoff

읽기 감사/실험 초안 진행 중. 현재 새GUI 결과를 본 뒤 실행 필요성과 게시 범위를 판단한다. 구현/측정 PASS가 아니다.

metrics_recovery가 /tmp/dlpad-windows-library-cold-open-cost.rs(e2581c5b6560abc3bb6b03a1f5f6677e9cca807d21c37b3994cbc134508955cc) 초안을 작성했다. 매회 fresh Store open/migrations·reconcile·128page의 wall/native creation-bound CPU를 분리하며 원DB HANDLE identity/currentpath·readonly10000행·reopenpage·sentinel을 확인한다. rustfmt만 실행했고 native 비용은 미측정이다. 현재GUI/패키지 source proof를 먼저 고정하므로 프로젝트게시·실행은 보류한다.

GUI/개발ZIP/S10가 원540source에서 끝났으므로 실험은 별도 evidence/experiment crate에게시한다. root가Cargo/lockcopy를조정하고offline exact 직접의존성/동일release s-thin-1-strip옵션으로빌드한다. productionCargo/lock/소스/CPU는그대로유지하며실험자체lock생성·Clippy·build·native실행을sourceCPUguard와함께직렬검증한다. 공개앱의캐싱정책을바꾸는구현은없다.

별도evidencecrate offline lock/Clippy/build/native실행4작업PASS, production540source/CPU불변·원Job자연0/private/input을root대조했다. freshopen wall346.2/225.3/210.7μs, scanCPU0.0625/0.046875/0.0625s, scanwall56.7/54.1/53.5ms였다. open/page CPU0ticks는counter분해능한계이며zero실작업으로해석하지않는다. readonlyDB integrity/10000행·sentinel·원DBID/page reopen도확인했다. 이표본에서DBopen은매우작아캐싱구현을채택하지않는다. GUI strictwindow의원인확정은아니다.
