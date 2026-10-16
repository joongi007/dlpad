# S02: cookie 소유 기록에 journal anchor 연결

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: root — cookies.rs, process.rs module 계약, 이 문서
- 선행 작업: reader journal create/reopen 실제 검증, owned-file cleanup API

## 사용자 결과

새 Linux session cookie의 정확 owner/record identity와 reader journal anchor를 하나의 durable recovery marker에 바인딩한다. 기존 record v1을 읽고 같은 boot는 계속 보존한다. 실제 I/B launcher gate가 연결되기 전에는 같은 boot cleanup 정책을 완화하지 않는다.

## 계약과 범위

Record optional journal anchor를 canonical digest에 포함하고 version2를 구분한다. journal 생성/record/marker sync 실패는 unknown copy 보존이다. cookie cleanup 전 실제 journal identity·proof와 전체 owner entry allowlist를 검증하고 fixed owned files만 제거한다. import/정상 cleanup/prior-boot recovery를 시험한다. 제품 reader 실행 gate·launcher receipt 생성은 후속 연결이다.

## 인수 조건

새 record2 anchor exact match, old v1 compatibility, mismatched/replaced/extra journal file 보존, pending reader 삭제 거절, source/sentinel 보존, bounded scan128 유지. 전체 engine 회귀·fmt·clippy 통과.

## 검증과 handoff

구현 중. source freeze 뒤 실제 Cargo 결과 기록.

Root 실제 cookie lib28개 PASS. 새record2 anchor 교체/unusedreceipt 보존과 기존v1 prior-boot 호환시험을 포함한다. engine all-targets clippy 진행 중. journal file 추가1개·최대receipt2개이며 새dependency/thread 없음. same-boot 회수와 제품I/B gate는 미완료다.

Root engine all-targets clippy 수정후exit0. 후속으로 active journal lock은 복구 보류로 구분하도록 수정하고 실제 same-boot live lease 보존회귀를 추가했다. 전체 engine 회귀 진행 중. Journal lock 보류는 이전 부팅/정리 증거를 대신하지 않는다.

최종 전체 engine exit0/library223 PASS(ignored2 별도), 모든 integration 및 strict all-targets clippy exit0. [독립 검토](../validation/evidence/cookie-journal-owner-integration/root-review.json)에 sourceSHA·원시로그를 기록했다. 제품 I/B gate·same-boot native receipt 복구는 별도 다음작업이다.
