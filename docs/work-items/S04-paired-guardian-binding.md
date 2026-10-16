# S04: 관리 실행·쿠키의 원 Job paired binding 순수 계약

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당: metrics_recovery
- 수정 파일: 신규 crates/engine/src/process/windows_paired_guardian_binding.rs, 이 문서
- 의존성: 원 ManagedReaderRecord/WindowsReaderRecord Bound schema; root module 선언 및 후속 paired runtime/config 연결

## API 및 순수 인수

PairedGuardianBinding::new는 두 원 typed record와 각각의 slot을 받아 pending Bound를 검증한다. from_intents는 두 원 Intent의 clone을 동일 actual guardian/tool creation FILETIME tuple로 bind하고 new를 재사용한다. Metadata와 cookie0, Download와 cookie1만 허용하며 VersionProbe/DirectProcessing 쿠키 attachment는 거절한다. getter는 각각의 원 typed Bound, creation tuple, pair digest를 반환한다.

두 원 record의 bounded encode/parse 검증을 재사용하여 anchors/session/record digest/각 nonce/slot/stored identity digest를 재계산 확인한다. 새 고정 domain, managed/cookie type tag, 두 index와 managed stage, 원 nonce/digest, 동일 creation LE로 SHA256 pair digest를 계산한다. 단독 managed/cookie의 digest로 pair를 대신하지 않는다. 고정 version/domain과 deny_unknown_fields serde attachment는 최대32KiB이며 경로/로그/handle을 포함하지 않는다. Debug/error는 고정 redacted 문자열이다.

회귀는 Metadata/Download 정확 mapping 및 roundtrip, 기존 단독 domain과 교차 decode 거절, creation mismatch/zero/slot범위/stage mismatch, anchors/nonce/digest/domain/unknownfield/잘린JSON/oversize를 포함한다. 이 순수 record에 실제 Job/receipt/file 권한은 없으며 anchor가 현재 파일과 일치한다는 OS 증거도 제공하지 않는다.

## 후속 수직 연결 경계

현재 download의 managed+cookie 경로는 중간 상태다. metadata/download가 기존 cookie helper로 빠지는 경로를 실제 paired runtime이 대체해야 한다. 하나의 PreparedChild/원 Tool Job에 두 원 journal을 각각 durable Bound→release하고 양쪽 independent lock/원receipt 권한 획득 성공 후에만 Begin/Resume해야 한다. 원 Job empty 후 두 receipt sync/lock release가 모두 완료돼야 pair Terminal을 인정한다. 기존 cookie record를 managed 삭제 권한으로 대체하거나 helper를 단순 중첩하지 않는다. startup 자동 회수는 현재 비활성 상태이며 이 pure 계약으로 완료를 주장하지 않는다.

## 검증 상태

개별 rustfmt exit 0. 담당은 Cargo/config/기존 schema를 수정·실행하지 않았다. root 실제 pure/native 통합 검증 후 그 범위만 추가한다.

## nested serde 및 원 파일 도메인 분리 보강

직접 decode뿐 아니라 다른 config의 nested serde 필드에서도 private wire→custom Deserialize→validate를 강제한다. 잘못된 원 Bound/anchors/digest를 가진 object가 getter까지 진입하지 않으며, 임의 nested deserialize로 순수 형식 검증을 우회하지 않는다. metadata/download와 유효 tuple만 생성한다.

cookie copy/journal/두 receipt가 managed marker/lock/journal/여덟 receipt 중 어느 원 native ID와도 같으면 거절한다. 동일 volumeSerial+fileID를 기준으로 비교한다. 부모 directory의 공통 ancestor나 root/cache directory ID 공유는 이 순수 계약에서 배제하지 않는다. 해당 형식은 실제 경로 포함 관계·ACL·원 파일 타입을 입증하지 않으며 각각의 guardian journal이 OS 검증해야 한다. directory 공유를 file 도메인 권한 대체로 해석하지 않는다.

직접/nested malformed parse 거절 및 cookie 네 file 위치의 managed marker alias 실패, directory 공유 형식 성공 회귀를 추가했다. 개별 rustfmt exit 0. 최종 source SHA256 `7f021da561890d3d0231e14c2445b95d5f2461be782e003ab8d509fe03a9acfb`로 동결 인계하며 actual Cargo/native 결과는 아직 없다.

주 담당 순수 계약 실행: cargo test -p dlpad-engine --lib process::windows_paired_guardian_binding --locked에서 4 passed / 0 failed. nested serde·원 파일 alias·creation/stage/nonce/domain 변조 거절을 확인했다. 단일 Job 두 receipt runtime 연결은 여전히 미완료다.
