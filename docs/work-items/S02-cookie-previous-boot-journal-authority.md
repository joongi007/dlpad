# S02: 이전 부팅 journal 삭제의 명시적 권한

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — cookies/reader_journal.rs 및 이 문서
- 선행 작업: root의 이전 부팅 복구 연결 계약

## 사용자 결과와 계약

이전 부팅의 원본 recovery marker에 결합된 boot/digest를 명시적 authority로 전달한 경우에만 pending Intent/Bound journal 삭제를 허용한다. authority는 생산 코드에서 bounded kernel boot UUID read로 현재 부팅을 캡처하고 각 unlink 전에 다시 확인한다. 기존 same-boot cleanup 정책은 변경하지 않는다.

PreviousBootAuthority는 private 필드이며 original boot와 record digest를 nonzero로 검증한다. journal의 모든 nonempty capability는 original boot/digest와 정확히 일치해야 한다. 원 directory/record/receipt ID, schema 및 Resolved actual proof를 계속 검증하며 proof를 생성하거나 위조하지 않는다. 빈 journal은 이전 부팅 권한에서 허용한다. partial deletion은 원자적이라고 주장하지 않는다.

## 인수와 검증

Intent/Bound 삭제, 이전 부팅 empty journal, boot/digest 불일치, 같은 부팅 authority 거절, malformed/oversize UUID, 교체된 native entry 및 Resolved proof 변조 보존을 시험한다. test-only synthetic current boot constructor는 생산 컴파일에서 제외한다. root가 cookies.rs 연결과 Cargo/native 인수를 수행한다. 자체 Cargo 및 schema/의존성 변경 금지.

## 구현 동결과 의미 구분

PreviousBootAuthority의 생산 new는 고정 kernel boot 파일을 최대 38바이트 읽고 정확한 UUID+newline/nonzero를 검사한다. 원 boot와 현재 boot가 같으면 권한을 만들지 않는다. 생산 revalidate는 매번 kernel 값을 다시 읽어 캡처한 현재 boot와 대조한다. synthetic override는 cfg(test) constructor에만 존재한다.

validate_previous_boot는 원 anchored directory/record/receipt와 canonical recordstate를 확인하고, 모든 nonempty 슬롯의 index/nonzero nonce/schema 및 boot/digest를 authority와 대조한다. Resolved 슬롯에는 실제 256바이트 full-match receipt가 계속 필요하다. 이전 부팅이라는 권한에서 Intent/Bound의 원 receipt 파일은 0~256바이트까지 허용한다. 이때 zero/partial receipt는 살아 있는 reader가 이전 boot를 넘어 존재할 수 없다는 별도 권한으로 정리하며, 종료 proof가 만들어졌다고 주장하지 않는다. partial/corrupt **recordstate**는 계속 오류이며 receipt의 미완성과 혼동하지 않는다. 256바이트 초과 receipt는 보존한다.

기존 cleanup_eligible 및 remove_owned_files는 same-boot 정책 그대로다. remove_owned_files_previous_boot만 명시 authority를 받고 기존 bound/unlink private 경로를 재사용한다. 각 unlink 전 현재 boot, 원 capability와 남은 file ID/record bytes/Resolved proof를 다시 확인한다. 파일 삭제의 부분 실패는 원자적이지 않으며 임의 receipt 생성/상태 resolve/foreign file 삭제는 없다.

unit 3개를 추가했다: 실제 kernel 및 bounded UUID parser/동일boot·zero·capture변경 거절, empty/Intent/Bound/Resolved0+Bound1 삭제, boot/digest/native 교체/Resolved 변조의 bytes 보존. 각 시험은 기존 공통 exact child 격리를 사용한다. Cargo 및 native 실행은 하지 않았다.

`.local-tools/cargo/bin/rustfmt --edition 2024 crates/engine/src/cookies/reader_journal.rs` 실제 exit 0. source SHA256 `1f8a2f16f1d0ac927f7d0487f7c09197f51a5eb6e3970a4d252a575da326fe1c`로 root 동결 인계했다. root cookies.rs와의 실제 컴파일/통합시험은 다음 인수다.
