# S01: 목록 worker 대기 간격과 Stop wake

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R01, R06, R08
- 담당/수정 범위: root crates/app/src/accessible_library.rs 및문서/native최종검증; diagnostic_cache /tmp500ms변경안전감사만
- 선행 작업: S00-bounded-identical-frame-raster-reuse 원GUI3회와threadwindow원FAIL확보

## 사용자 결과

아무명령·원인이없는목록worker의watch확인빈도를낮춰idle비용을측정한다. 명령채널은즉시깨고Stop은기존bounded채널의wake-only명령으로알린다. 실제성능개선은아직미검증이다.

## 계약과 범위

quiet recv_timeout만100→500ms, scan중100ms checkpoint/CPUquery/wait·원180초deadline·30초periodic/1초ordinary/2초retry 내부gate는유지한다. 일반watch전달의poll추가지연상한은100→500ms로변경되며무조건1초UI반영보장은없다. AtomicStop을Release로먼저설정하고기존capacity1채널에StopWake를try_send한다. full이면이미깨울명령이있고disconnect는종료경로이며blocking/새채널/thread/mutex는없다. wake-only는root선택/스캔/DB/persistedstate/serialACK를만들지않는다. worker는recv후Stopbit를검사하고소유자원drop후ACK한다.

## 인수 조건

StopWake free/full/disconnected 실패경로, 실제idleStop/즉시Next/scanStop rollback·ACK·source보존 회귀. 기존window3 library78.125ms의idle/scan분해원인은미확정이고측정근거없이해결로표현하지않는다. 최신whole4/GUI3x60 avg/원strict10s1%/80MiB 및실제Next/Stop/180초완료·자연Job0를root검증한다.

## 검증과 handoff

readonly500ms안전감사는poll전달지연/원AtomicStop최대500mstradeoff를지적했다. root는기존capacity1채널wake로Stop대기지연을피하는최소변형을통합할예정이며아직제품변경/native시험전이다. Cargo/schema/외부데이터변경0.

root가기존단일파일에quiet500ms/StopWake를통합하고channel free/full/disconnected3실패회귀를추가했다. Nativefmt는exit0, 현재whole4인수중이다. read-only 통합감사는등록전후/full/disconnected/scanrollback/소유자원drop→ACK/DB와serial비처리경로에서차단결함없음으로보고했다. root가직접통합검증하며이는native성능PASS나idleStop인수대체가아니다.

Windowswhole4/root현재source543/default3ca7df..7917568B及frame3/StopWake3回귀PASS. GUI60초3회avg0.364343/0.260065/0.389964%, max10초0.625598/0.623536/0.779698%, memory全PASS. root18창原수식/currentpins/原Jobguards와독립감사대조PASS이며quietHost는미측정이다. 原3만payload·DB 검토/현재fixtureClippy 및실제완료·Next·Stop·rendererpixel을이어간다. nobank純cpu候補/tmp는미실행·미채택으로현재CumulativeBudget를변경하지않는다.

root가현재3회原3만payload/CAP·3개DB각1만행/정확membership/size/present/identity형식/무결성보존을全확인했다. fixture featureClippy/앱회귀도PASS다. 최신소스의실제completion native를새evidence에서실행중이며Source543동결을유지한다. Next/Stop·pixel이끝나기전scoped완료로승격하지않는다.
