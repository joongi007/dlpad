# S02: Windows reader 기록 공유 계약

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: root — cookies.rs 및 cookies/windows.rs module 연결, 이 문서; metrics_recovery — 신규 cookies/windows_records.rs 및 별도 workitem
- 선행 작업: Windows sameboot receipt 구현 감사와 기존 private importer

## 사용자 결과

Windows 시작 복구를 위한 durable 두 reader 상태와 완료 receipt 계약을 구현한다. 이번 단계는 순수 포맷/전이 기반이며 실제 회수 완료는 guardian/native journal/public recover 연결 뒤 인수한다.

## 계약과 범위

Windows 전용 v1 포맷. 기존 Linux 포맷/권한은 변경하지 않는다. 최대4096B JSON record, 두 일회성 슬롯. session nonce16B와 원 record digest32B, cache/owner/copy/journal의 original Windows volume_serial+file_id anchors를 고정한다. nonce/digest/file_id는 nonzero, 모든 identity는 Windows variant여야 한다. 전체 Debug/오류는 경로/nonce/identity를 생략한다.

Empty→Intent→Bound→Resolved. download1은 metadata0 Resolved 뒤만 가능; 슬롯 재사용 금지. Bound는 slot별 nonce16B 및 원 guardian/tool creation FILETIME u64를 포함하며 숫자PID/JobHANDLE은 durable 권한으로 쓰지 않는다. canonical identity digest는 domain separator와 session anchors/slot/capability/native creation identity 전체를 해시한다. 해당 process creation은 원 volatile handle에서 검증하는 실행 시 evidence이고 startup 단독 삭제 권한이 아니다.

256B 이내 고정 binary completion receipt는 version/index/session nonce/record digest/bound identity digest 및 완료 magic으로 정확한 길이·reserved0·digest를 검사한다. 원 guardian의 write-only capability와 native FileId/ACL/lock을 검증한 I/O 계층만 실제 receipt 권한을 만든다. pure parser는 arbitrary file을 trustedreceipt로 만들지 않는다. 다른 boot 회수 추정 필드 없음. JSON unknown/oversize/잘못된variant/partialreceipt/mismatch와 invalid transition 모두 거절한다.

## 인수 조건

순수 실패 경로 시험: illegal order/reuse/extra字段/oversize/Unixidentity/zeroanchors/mismatched identity/partial receipt/foreign digest. 정상 두reader의 정확 receipt만 resolve. 실제 Windows compile/runtime은 root 후속 검증.

## 검증과 handoff

shared schema 변경 root 조정. 새 Cargo dependency 없음; serde/sha2 기존 의존성. 최대4KiB record/256B receipt 및 bounded 두slot 메모리 비용. 제품recover는 이 포맷 기반만으로 활성화하지 않는다. 후속 Windows journal/native lock/guardian 및 app crash fixture가 필요하다.

## Journal 연결 전 후속 공유 계약

Root가 원래 receipt0/1 native FileId도 원세션에 durable바인딩하도록 결정했다. SessionAnchors에 receipts:[WindowsAnchor;2]를 추가하고 canonicaldigest에도 두anchor를 포함한다. Journal생성시 두receipt를 create-new/protectedACL로 미리 만들며, 원immutable recoverymarker와 초기journal이 모든anchor를 저장한 뒤에만 Intent/Bind/Begin을 허용한다. 이후 Begin은 원래 pinned빈receipt를 사용하고 현재관측FileId로 대체하지 않는다. 기존6개 순수시험 snapshot과 이 후속보강은 별개이며 후속source는 전체검증 terminal뒤 통합한다.
