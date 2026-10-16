# S01: 활성 다운로드의 부분 파일 이벤트 비용 감사

- 상태: done
- 분류: performance
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: soak_recovery_audit /tmp 읽기감사·재현/test초안; root 게시·native시험·통합검증·본 문서
- 선행 작업: 현재목록전송통합·최신전체/GUI/S10검증

## 사용자 결과

부분 다운로드가 목록에 노출되지 않는 동안 불필요한 목록scan이 실제로 발생하는지 확인한다.

## 계약과 범위

watch의 non-access dirty와 library partial 제외 및 download publication을 읽기대조한다. 재현을 먼저설계하며일반 `.part`파일을권한있는cache로삭제하지않는다. 후보최적화는정확reservedsubtree아래의명확한partial-only힌트만고려한다. mixedrename→최종파일/emptyunknown/backenderror/rootnamespace 변화는보수dirty,기존30초재확인·rootmtime는유지한다. sourcefreeze중프로젝트/native수정0.

## 인수 조건

실제synthetic nativeevent/scan빈도근거·최종rename/publicfile변화반영·mixed/empty/모호paths 실패경로·메모리상한·원DB파일보존. 원인미측정이면성능결함확정이나최적화완료로표시하지않는다.

## 검증과 handoff

읽기감사를시작했다. 구현/실험은후속root판단이다.

실제publication이rename뿐아닌root hard-link/Create라는읽기감사결과를반영했다. publicWatcher와별도boundednotify observer를사용하는/tmp초안을별도evidencecrate에게시했다. 원canonical partialpath의3번분리append+sync→실제partialonlyevents와제품dirty→10000불변scanCPU를관측하고,최종hardlink→10001행/12B/native128ID 및원10000bytes/sentinel을검증한다. root가Cargo/lockcopy/offlineexactdep를조정하며production540/CPU는유지한다. actual원문제재현전suppression을채택하지않는다.

원baseline native3회에서partial-only nonAccess→제품dirty→10000불변scan이실제관측됐다(CPU0.046875/0.078125/0.078125초). final hard-link/Create→10001행/12B/original128ID와원10000파일바이트보존도PASS. 그러나partial과ordinary가hardlinkalias면partial경로수정이ordinary내용을바꿀수있어lexical subtree만으로힌트를억제하지않는다. 별도alias counterexample을evidencecrate에게시해원publication12B와후속alias13B/실제eventpaths를구분측정한다. production정책변경0이다.

alias actual실험도4작업PASS/원source·CPU불변이었다. ordinary12→13B/mtime변경/DB13B/동일원128ID를검증했으나partial와ordinary이벤트가모두나와이호스트의partial-only반례는미확정이다. root가두실험Job/source guard·readonlyDB integrity/10001행·원partial/finalbytes/sentinel을대조해root-review.json을보존했다. 검토보조의중복snapshot루프를이번작업원PID/start로확인해종료(exit143)하고한번만수집하도록고친재검토는PASS. native제품시험종료와는별개다. 이후watch정책이변경된새source는별도검증중이다.
