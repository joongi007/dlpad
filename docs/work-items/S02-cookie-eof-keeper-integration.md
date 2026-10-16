# S02: 제품 EOF keeper와 gated parent 수명 통합

- 상태: ready (소스 freeze; 실제 통합 검증 대기)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery; process/user_service.rs 및 이 문서
- 선행 작업: EOF holder root 독립6case PASS, root guardian 내부keeper marker 계약

## 사용자 결과와 계약

reader RAEfalse와 별도 app.slice EOF keeper로 초기 identity 확보 전 오류에서 active(exited) 잔여물을 피한다. pipe2CLOEXEC writer는 launcher만 소유한다. guardian marker 구현은 root 담당이며 신규 schema/Cargo/다른소스 변경 없음. keeper fixed helper/추가args0/readFDstdin/devnulloutput; Requires/Afterparent, readerWants/Afterkeeper. receipt sync 전후 keeperalive/nativeparent identity를 확인한 뒤 EOF→keeperterminal/Unref→readerrelease→parentUnref. 부모 직접Stop0, nongated 유지, 제품RuntimeMax 추가 없음.

## 인수 조건

원 reader 종료값/pidfd/strict parent held+freshempty 유지, keeper exactInvocation/pidfd/nativeoutsideparent, writer상속0, receipt없으면삭제승격0. success/failure/signal/cancel/appworkerkill 및 acceptedstart 초기unknown failure 자연GC와 foreignqueued보존 실제 인수는 root 담당이다.

## 검증과 handoff

작성 뒤 소유 rustfmt와 sourcefreeze. 자체Cargo/native 실행 금지.

## 소스 인계

pipe2(O_CLOEXEC)의 fresh read/write OwnedFd를 launcher에서 구성하고 manager에는 keeper read-end만 전달한다. StartTransientUnit reply 뒤 local read-end를 닫는다. GatedStart private 구성으로 same transaction primary reader Wants/Afterkeeper 및 auxiliary SWUparent/keeper를 encoding한다. reader는 RAEfalse/noignorefailure, keeper는 fixed현재helper+marker only, app.slice/RequiresAfterparent/RAEfalse/Delegatefalse/stdoutstderr O_RDWR devnull이다. 생산 RuntimeMax는 추가하지 않았다.

keeper exact Invocation과 seed MainIdentity/pidfd를 capture하고 /proc membership 및 canonical cgroup native pin을 확인한다. receipt 전후 fresh PinnedCgroup identity 비교, 원 keeper livePID/Invocation/Delegate/Slice와 parent 밖 membership을 요구한다. 기존 reader 원 종료값/pidfd 및 parent strict native empty, durable sync를 유지한다. 정상 receipt 뒤 writerEOF, keeper exit0 및 pidfd 종료/Unref, reader release, parent Unref만 수행한다. unknown identity는 이름추정Stop이 없고 OwnedFd/bus/control lifecycle에 맡긴다. known serviceguard는 기존 exact cleanup 권한만 사용한다.

소유 rustfmt 실제 PASS. frozen user_service source SHA256 `4714dc6d9272dd335ba36ef37fc8476a4ebcc5accbee4e0d4238a52640a54196`. Cargo/native 자체 실행 없음. root guardian marker와 함께 실제 helper compile 및 cookie native/service/download/recovery/초기resourcefail/외부queued 인수가 필요하다. 인계 뒤 읽기 검토에서 wait_unit_seeded는 keeper가 EOF뒤 active에 남으면 전체 종료deadline이 없을 수 있음을 root에 전달했다. 종료관측5초 상한 보강은 root freeze 합의 후 수행할 예정이며 아직 구현했다고 주장하지 않는다.

root가 Cargo 미착수 상태에서 EOF 종료 상한 보강을 승인했다. keeper writer drop 후 CLEANUP_GRACE5초 absolute deadline 안에 exact Invocation과 inactive/failed 상태 및 original pidfd not-live를 함께 확인하고 기존 seeded terminal 원 종료값 검증으로 진행한다. active tool 실행 timeout은 변경하지 않았다. 소유 rustfmt PASS, 최종 frozen SHA256 `e0b6ea936d5f518e0a90d805ca91144b2c45fdc9cd8acf1632f699c07b7d0439`. 자체 Cargo/native 실행은 없다.

## 초기 startup failure 실제 시험 연결

root가 keeper typed Delegate boolean getter 및 Starting Invocation capture 보강 후 helper/native service3+download4+recovery2 exit0을 보고했다. 해당 source 변경은 root 담당이며 아래 담당 debug hook 변경과 구분한다. 최신 scope는 gated-only debug 환경 DLPAD_TEST_COOKIE_UNIT_NONCE를 exact32 ASCIIhex/nonzero UTF8로 검증해 lowercase canonical service/keeper/parent nonce를 고정한다. path/잘못된길이/nonhex/NUL/nonUTF8를 허용하지 않는다. release는 이 env를 조회하지 않으며 nongated nonce는 기존 OSrandom을 유지한다. unitmodefail과 기존 소유 Stop 권한은 변경하지 않는다.

gated after-start fault는 manager accepted reply 후 passed control/keeper read-end localdrop 직후, after-get-unit는 get_unit 뒤 Invocation capture 전, after-invocation는 원 reader RAII 구성 뒤에 삽입했다. 기존 debug-only 정확 fault selector와1초 합성 관측 창을 재사용하며 release 동작은 없다. 순수 nonce parser unit은 경계/nonUTF8/uppercasecanonical을 시험하며 globalenv를 바꾸지 않는다.

소유 rustfmt PASS, frozen source SHA256 `40713e1bba54e124a6448e745730d6446884e7303f6e5f1e31689a32c6be0c0b`. Cargo/native 미실행. root와 metrics의 신규 초기 resourcefail 실제 fixture에서 tool0/knownPID정리/unknownidentity추정Stop0/foreignqueued보존/FDbaseline을 확인해야 한다.
