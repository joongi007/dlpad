# S04: 검증 ZIP bounded component 추출

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R11
- 담당/수정 범위: diagnostic_cache — crates/engine/src/tools/bounded_zip.rs 및 이 문서
- 선행 작업: root ArchivePackage/ArchiveComponent, rawzip0.5.1/flate2 exact Cargo pin

## 사용자 결과

SHA256와 크기가 검증된 ZIP에서 manifest component 하나를 고정 버퍼로 추출한다. 파일 capability는 caller 소유이며 source 변경/삭제, 경로 생성, 실행·mode·정리는 하지 않는다.

## 계약과 범위

extract_verified_zip(&File,&ArchivePackage,component_name,&mut File,&AtomicBool,Instant)->Result<u64,ToolError>. archive hash 전체 exact streaming 검증 후 rawzip parser/lending directory iterator 사용. entry<=32·hint exact·flatASCII 이름·중복/암호화/symlink/special/unsupportedmethod/localcentral mismatch/overlap·미선언component 거절, 총expanded256MiB. stored/deflate 고정버퍼 추출 CRC/actualsize/componentSHA 확인, cancel/deadline. output 초기len0/offset0. Cargo/sharedtypes는 root 담당.

## 인수 조건

실제 stored/deflate fixture 정상 대조 및 hash/CRC/경로/중복/entry폭주/크기폭탄/취소/deadline/출력재사용 실패 시험. source bytes 보존, 부분 output은 caller 처리.

## 검증과 handoff

구현 중. [rawzip0.5.1 공식 문서](https://docs.rs/rawzip/0.5.1/rawzip/)와 ZipEntry local_header/verifying_reader API 확인. 공식 security 계약상 count/duplicate/bomb/overlap과 filesystem ownership은 consumer 책임이다. [ZipEntry](https://docs.rs/rawzip/0.5.1/rawzip/struct.ZipEntry.html). 제품 installer/UI 전체 완료 범위가 아니다.

실제 담당 파일 rustfmt exit0. `cargo test -p dlpad-engine --lib tools::bounded_zip --locked` 최종 7PASS/0failed/0.06초(compile17.02초). 정상 stored/deflate bytes와 source 보존, archive/component SHA mismatch, path/duplicate/33entries, CRC/local mismatch/symlink/encryption/unsupported method, data descriptor 세 필드변조, trailing compressed garbage/count hint mismatch, expansion lie/cancel/deadline/nonempty output 거절을 확인했다. rawzip Verifier는 actual 중앙CRC/size를 검증하고 별도 data_descriptor 대조 및 DeflateDecoder total_in 전체소비 검사를 추가했다. compressed뿐 아니라 original localheader start부터payloadend 범위중복과 directory 영역 겹침을 거절한다.

첫 회귀는4PASS/1FAIL이었다. rawzip writer가 ../deno를 deno로 정규화해 “악성 ZIP” fixture가 공격 내용을 포함하지 않았다. 기존 결과를 성공으로 쓰지 않고 local/central 실제 filename bytes 변조로 수정한 뒤 5PASS→descriptor6PASS→최종7PASS를 확인했다. Cargo/shared파일은 수정하지 않았다. root 공식 archive 실제 추출/RSS 및 workspace/3OS 통합 검증은 별도다. parser 전달 source/output의 native provenance·동시 수정 방지는 caller 책임이며 이 함수는 삭제·publication 권한을 갖지 않는다.
