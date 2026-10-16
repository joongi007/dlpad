# S01: 연속 파일 이벤트의 재확인 합치기

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root crates/engine/src/watch.rs·CONTEXT·ADR·전체/nativeGUI검증·본 문서; metrics_recovery /tmp 계약·회귀초안읽기검토
- 선행 작업: 실제partialdirty/10000불변scan 비용·hardlinkalias 이벤트안전감사

## 사용자 결과

연속 파일 변경이 몰려도 일반 이벤트로 인한 전체 목록 재확인은 초당 최대1회로 합치고 마지막 변경을 남김없이 반영한다. 사용자 수동 명령은 합치기 지연 없이 처리한다.

## 계약과 범위

경로별힌트를버리지않는다. worker가소비한ordinarynative/rootmtime dirty를상수크기pending+nextdue로유지하며다음1초deadline에합친다(실제poll·진행중scan비용은별도). 최초watch부착·backend오류/rootidentity손실·health복구·30초보완은즉시재확인하며pending을정리한다. 기존rootmtime의30초deadline갱신·취소/StopACK/DB권한/파일삭제권한/명령ACK는유지한다. API/schema/deps/Cargo변경0. 파일IO중새이벤트는기존atomicflag에보존된다.

## 인수 조건

연속힌트중scan폭주상한·마지막pending보존·정확deadline·idlebusy없는timer·urgentfailure/reconnect/periodic우회·root/ordinary/finalCreate 반영을회귀검증한다. native합성burst와원DB/files보존·최신전체4명령/목록GUI/S10통합을실행한다. 주기CPU원인이나실제다운로드빈도전체를단정하지않는다.

## 검증과 handoff

계약독립검토를시작했다. 기존actualpartialbaseline는매write별scan을재현했고alias실험은ordinary이벤트도관측되어이호스트의partial-only반례는미확정이다. lexical필터는채택하지않는다. 현재코드구현은후속이다.

root가OrdinaryHintGate의fixed next_due/pending을실제WatchSession에통합했다. 일반native/rootmtime만합치며health/backend/rootidentity/periodic는urgentflush다. poll100ms와inflightIO를포함한DBcommit≤1초하드보장은주장하지않는다. pure3시점회귀와실제session rootmtime pending/백엔드실패/재부착 우회2회귀를추가했다. 아직native컴파일/전체검증전이다.

실제Windows watch9회귀·library27회귀·app두후보각14회귀·fixture포함전체Clippy PASS(windows-watch-hint-coalescing-20261005-continue). source guard/원Job자연0/private/input을원log로확인했다. public WatchSession 12×250ms burst/5초flush/ordinaryCreate+alias13B 반영의별도evidencecrate를root게시했다. native실측후최신defaultCPU전체4명령/GUI를재실행한다.
