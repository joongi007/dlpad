# S04: Bounded TAR stream reader foundation

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: diagnostic_cache; crates/engine/src/tools/bounded_tar.rs, 이 문서만
- 선행 작업: root API 합의 및 module export; XZ decoder/installer 별도

## 사용자 결과

고정 FFmpeg 공급의 압축해제된 TAR를 전체 메모리 또는 외부 tar 실행 없이 읽고 선택 component만 caller 소유 빈 File에 검증 작성하는 기반이다. 관리 배포 채택 완료는 아니다.

## 계약과 범위

공식 후보 실제 inventory는 73개 항목, 선언 450209477 bytes, ffmpeg/ffprobe/license 선택 합계284847347 bytes이다(docs/validation/ffmpeg-supply-probe.json 및 ffmpeg-supply-validation.md). 기존 ArchivePackage256MiB 제한보다 크므로 별도 reader limits를 명시하며 installer 기존 정책을 변경하지 않는다. source는 caller Read stream, 압축 archive SHA/XZ memory limit은 caller 책임. 직접 TAR header512/checksum/strict octal/USTAR name+prefix를 검증한다. path root/중복/link/special/선택 SHA-size/truncation/padding/end marker/trailing 제한을 확인한다. 외부 tar, shell, extractall, 전체 stream allocation, source 변경, 파일 경로 생성/삭제는 없다.

## 인수 조건

합성 regular+directory와 prefix/mapping 정상, checksum/size/path/duplicate/link/special/malformed/truncation/nonzero padding/tail/entries/total limit/cancel/deadline/hash 실패를 실제 unit fixture로 검증한다. 고정 unselected regular는 bounded discard하되 header/path/type 검증을 생략하지 않는다. POSIX USTAR와 실제 원 공급의 GNU ustar-space magic을 지원하며 GNU345..512 reserved영역은 전부zero를 요구한다. GNU longname/sparse 및 PAX는 명시 거절한다. 실제 공식 tar header format은 root 후속 실증으로 확인한다. 부분 실패 output의 보존/삭제는 caller 책임이다.

## 검증과 handoff

TarExtractionRequest(root/mappings/selected/limits)와 extract_tar_component(Read stream, request, output File, cancel, deadline)를 구현했다. 4개 unit fixture 그룹을 작성했으며 컴파일/실행은 root 통합 대기이다. 단독 rustfmt PASS; Cargo/root module은 수정하지 않았다. XZ 라이브러리 및 실제 FFmpeg 설치/3OS 공급은 미검증이며 후속 범위다.

root는 tools module/public export와 shared component_name의pub(super)노출을통합했다. `cargo test -p dlpad-engine --lib bounded_tar --locked`4개그룹PASS. 첫all-targetClippy는시험fixture의octal-looking NUL literal 경고로exit101이며같은8byte POSIXmagic을명시hexNUL로교정했다. 실제제품parser정책변경없으며후속test/Clippy진행중이다.

교정후 scoped TAR4개그룹재시험과 engine all-target Clippy 모두terminalexit0. root는실제성공output바이트assert와실패fixture/소유임시file종료뒤삭제를읽기검토했다. root fmt전체적용을완료했다. XZdecoder는아직workspace의존성이아니며공식tar.xz제품추출·Windows·전체workspace는후속이다. 현재원archiveheader관측은GNU일부헤더의실증일뿐전체TAR성공이아니다.
