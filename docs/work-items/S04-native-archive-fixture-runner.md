# S04: 네이티브 archive fixture 검증 runner

- 상태: in-progress
- 분류: test
- 우선순위: P0
- 요구사항: R03, R04, R11
- 담당/수정 범위: metrics_recovery — crates/engine/examples/verify_managed_archive.rs 및 이 문서만
- 선행 작업: root ArchivePackage schema와 별도 담당 bounded_zip 구현

## 사용자 결과

기존 실제 archive를 read-only로 열어 manifest에 고정된 한 component를 익명 tempfile에 bounded 추출하고 크기·SHA 및 source 불변을 실제 native runner로 검증한다. 설치·실행·publish는 하지 않는다.

## 계약과 범위

CLI 인자는 archive 절대경로, manifest 절대경로, component 이름 세 개다. manifest는64KiB 이내 serde JSON decode 및 validate를 통과해야 한다. source hash 전후와 manifest archive hash/size를 대조한다. extract_verified_zip 호출은120초 deadline과 AtomicBool을 전달한다. 익명 tempfile sync/seek/stream hash로 반환 크기 및 component hash를 다시 검증한다. JSON 출력은4096바이트 이내 고정 필드이며 path·자격 증명이나 raw 오류를 출력하지 않는다. 오류는 고정 redacted JSON과 exit2다. Linux VmHWM은 bounded /proc/self/status에서 읽고 불가시 null이다. 해당 high-water는 runner 전체 비용이며 추출만의 incremental RSS로 확대하지 않는다.

## 인수 조건

root의 실제 Deno ZIP+manifest로 source 전후 동일과 정확 output hash/bytes를 확인한다. 잘못된 manifest/size/hash/member는 고정 오류 exit2이며 외부 source나 설치 경로를 수정하지 않는다. archive tool 실행/chmod/publish/install/기존 파일 삭제는 없다. tempfile은 소유 임시 파일만 Drop으로 닫는다.

## 검증과 handoff

source 구현을 게시했다. Cargo/schema/다른 module을 수정하지 않았고 compile/native는 bounded_zip READY 이후 root가 수행한다. 현재 실제 archive 성공을 주장하지 않는다. 출력 elapsed는 추출+검증 구간이며 source initial hash와 manifest decode는 제외된다. 전체 CPU/RSS benchmark는 root 측정에 한정한다.

## 독립 parser 읽기 감사

검토 snapshot bounded_zip.rs SHA256은35aca2bdc40a57cc0757a992b83dfc35cd188f1cf4eb316082e1cdeafc064e3e다. 별도 담당이 계속 수정 중이므로 이후 코드 전체 검증 결과로 확대하지 않는다. source 수정/Cargo/native 실행은 하지 않았다.

- DataDescriptor: cached rawzip0.5.1 archive/reader.rs308·439의 verifying_reader는 central CRC/크기만 검증한다.487의 reader.data_descriptor()는 별도 호출 API다. 현재 descriptor flag가 있으면 local CRC/크기 비교를 생략하고 실제 descriptor를 읽지 않는다. 따라서 descriptor CRC/압축·확장 크기 변조를 명시 거절하는 계약이 아직 충족되지 않는다. 각 entry의 실제 descriptor를 읽고 central exact fields와 대조하는 최소 보강 및 중앙 hash를 재계산한 변조 fixture 회귀가 필요하다.
- 크기 상한: source metadata와 stream actual bytes, archive SHA, central entry count≤32, 선언된 component 크기/전체 확장≤256MiB, 실제 selected output 크기 및 SHA가 중복 검증된다. deflate total_in==compressedrange와 verifying_reader EOF CRC/size는 trailing bytes 거절에 도움이 된다. 독립 trailing-deflate fixture 실제 실행은 미검증이다. selected 이외 member는 선언/구조만 읽으며 실제 decompress/CRC/hash 검증 완료로 보고하면 안 된다.
- local/central 범위: 이름/flags/method 대조 및 압축 데이터 overlap/central directory 이전 범위가 있다. 현재 ranges는 압축 데이터만 포함하므로 다른 entry의 local header/descriptor를 덮는 구조 overlap 전체를 거절한다는 주장은 아직 없다. full structural interval 검증을 별도 좁은 반례 fixture로 확인해야 한다.
- capability alias: 비어 있고 position0인 output 및 양수 archive 크기 검사로 source와 동일 inode output은 통상 거절된다. 그러나 API는 regular output/쓰기 모드/독립 native FileId나 O_APPEND를 명시 검증하지 않는다. 호출자는 새 독립 tempfile 및 exclusive ownership을 보장해야 한다. 외부 concurrent writer/source 변조 방지는 parser의 약속이 아니다. example은 독립 익명 tempfile과 source 전후 hash를 사용한다.
- 실패 partial output: CRC/hash/쓰기 오류는 이미 쓴 prefix를 남길 수 있으며 parser가 truncate/unlink하지 않는다. 이는 caller 소유 partial cleanup 계약과 일치한다. publication은 성공 뒤 별도 원 identity 검증에만 허용한다. source path를 다시 열어 delete하지 않는다.
- 취소 응답: hash 및 decode 반복의64KiB 경계와 write 전에 cancel/deadline을 확인한다. source Read/output write_all/rawzip entry scan/개별 inflate 호출 내부 blocking을 interrupt하지 않으므로120초는 hard real-time syscall cap가 아니다. caller native local regular file 조건과 bounded entry32 조건을 유지해야 한다. 현재 pre-cancel/pre-expired 시험은 실행 중 취소 responsiveness 증거가 아니며 별도 중간취소 회귀가 필요하다.

현재 증거를 근거로 DataDescriptor 검증 부재는 구체적 보강 대상으로 root에 보고했다. 다른 항목은 명시 API 경계 및 미검증 반례이며 실제 결함 재현으로 확대하지 않는다.
