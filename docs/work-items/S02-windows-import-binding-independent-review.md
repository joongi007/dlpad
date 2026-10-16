# S02: Windows import binding 독립 읽기 검토

- 상태: completed (편집 중 source 감사; native 통합 인수 미완료)
- 분류: validation
- 우선순위: P0
- 요구사항: R02, R08
- 담당/수정 범위: soak_recovery_audit — 이 문서만
- 선행 작업: diagnostic_cache의 windows.rs import binding 구현; root source freeze

## 읽기 범위와 확인한 계약

현재 windows.rs OwnedCopy/import/cleanup, windows journal 및 recovery_marker, cookies.rs legacy reader_pending을 읽었다. 제품 source가 동시 편집 중이므로 line 번호나 중간 상태를 최종 source 결함으로 확정하지 않는다. Rust/Cargo/fmt/build/native 실행은 하지 않았다.

import는 source를 합성/일반 source read 계약으로 읽고 private SID DACL의 랜덤 owner directory를 만든다. owner 원 handle은 FILE_SHARE_READ|WRITE만 허용하여 namespace rename/delete를 막고 유지한다. copy는 SHARE_ALL로 도구의 정상 read/write를 허용하지만 retained original ID와 현재 path ID, disk/nonreparse/single-link, SID ACL을 다시 검증한다. 외부 source와 cache 조상은 삭제 대상이 아니다.

초기 write_owned_copy 완료 후 initialization_started를 설정한다. journal/marker 생성 도중 실패하면 defer_cleanup으로 handles를 닫고 부분 산출물을 보존한다. 완성 이후 tool 검증은 다섯 OWNER_ENTRIES만 정확히 허용하고 marker 원 handle/현 path/native anchors/session nonce/base digest와 independently validated journal record를 결합한다.

cleanup은 모든 삭제 대상의 DELETE handle을 열어 ID/ACL/kind를 먼저 검사하고 최종 validate_for_tool 뒤 원 handle disposition을 수행한다. cookie/journal/두 receipt/marker 이외의 파일을 재귀 삭제하지 않는다. owner directory는 원 handle 검증과 실제 빈 디렉터리 확인 뒤 disposition한다. 변경·hardlink·extra entry는 보존한다. journal all_receipts_match는 Empty/Intent slot의 unused receipt가 비어 있음을 검사하므로 빈 journal에 foreign receipt payload가 있는 경우도 성공으로 삭제하지 않는다.

cookies.rs public cleanup/Drop는 legacy reader_pending이 true이면 먼저 defer_cleanup하고 ReaderUnconfirmed를 반환/보존한다. Windows 새 durable binding이 생겼다는 이유로 이 기존 직접 engine reader 보호를 제거하면 안 된다. 새 native guardian pending journal과 legacy AtomicBool은 서로 다른 증명이며 한쪽을 다른 쪽의 완료 증거로 대체하지 않는다.

## 통합 담당에게 전달할 실제 인수 지점

부분 disposition 실패는 transaction이 아니다. 현재 순서는 cookies.txt부터 삭제하므로 이후 journal/receipt/marker disposition 실패 시 일부 파일이 이미 삭제될 수 있다. Drop의 재cleanup은 strict allowlist/현재 ID 검증으로 추가 삭제를 거절하고 남은 entry를 보존한다. 이것을 전체 원상복구 또는 모두 삭제 성공으로 보고하면 안 된다. recovery scanner가 incomplete namespace를 유지하며 사용자에게 고정 recovery 오류를 주는지 별도 실패 회귀가 필요하다. 이미 쿠키 copy가 지워진 상태에서는 신규 reader 재사용도 거절해야 한다.

삭제 전 preflight 뒤 entry가 추가되는 경우 original owned files만 disposition하고 directory empty 검사에서 실패하는 정책을 명확히 유지한다. 외부 entry가 남아 있다는 이유로 recursive cleanup하거나 ACL을 넓혀 재시도하지 않는다. marker digest가 일치한다는 사실만으로 native handle/lock/allowlist/receipt 검사를 생략하지 않는다.

GuardianJournal unlock/reacquire와 marker/journal binding은 root helper product wiring과 함께 실제 Windows fixture로 확인해야 한다. 새 import 기본 owner entries가 한 개에서 다섯 개로 바뀌어 기존 Unix 가정 또는 Windows fixture의 count assertion이 있다면 scope별 갱신이 필요하다. 실제 source 편집 담당에게 부분 삭제 인수와 namespace share 정책을 공유했다.

## 잔여

최종 source freeze 후 native 정상 import/tool access/cleanup, unused receipt payload, changed ACL/ID/hardlink/extra entry, journal initialization failure, 부분 disposition failure 및 legacy pending Drop 보존을 통합 담당이 실행해야 한다. 이 문서는 native PASS 또는 startup Windows recovery 채택 완료를 주장하지 않는다.
