# S01: Windows handle 기반 native file identity

- 상태: done
- 분류: reliability
- 우선순위: P0
- 요구사항: R03, R08, R11
- 담당/수정 범위: library_identity — crates/engine/src/file_identity/windows.rs, 전용 검증 문서/로그
- 선행 작업: Windows GNU native CLI, 주 담당의 file_identity.rs enum/모듈 공유 계약

## 사용자 결과

Windows에서도 실제 파일 ID를 관측하여 이름 변경과 공개 복구에서 같은 파일인지 확인할 수 있는 근거를 제공한다. 경로·크기·시간·내용으로 파일 ID를 추정하지 않는다. 사용자에게 연결되는 library/publication 통합은 주 담당이 수행한다.

## 계약과 범위

from_file(&File) -> io::Result<NativeFileIdentity>는 FILE_ID_INFO의 volume serial u64와128bit file ID의16byte 배열을 반환한다. enum/schema/serde 및 호출부는 주 담당 소유다. pub(super) open_regular_no_reparse(path), final_path(file), ensure_regular_under(rootFile,fileFile)의 Windows helper만 추가한다. from_file은 path containment 증명이 아니다. ancestor/path 변경의 모든 TOCTOU를 해결했다고 주장하지 않는다.

기존 windows-sys FileSystem feature로 지원하며 새 dependency/런타임/tool 설치를 추가하지 않는다. 기존 library/identity.rs, download/publication.rs, download.rs, store/app/Cargo/lock 수정은 금지한다.

## 인수 조건

- 실제 Windows temp 파일에서 hardlink/rename의 identity 유지와 replacement의 identity 변경을 확인한다.
- 열린 handle은 pathname replacement 뒤에도 원래 파일 identity를 유지한다.
- 디렉터리/비disk handle을 regular helper로 받아들이지 않는다. 경로 길이와 final-path 버퍼를 제한한다.
- root 및 final-file handle의 실제 최종 경로와 볼륨을 관측하여 외부 경로를 거절한다.
- 가능한 경우 temp symlink/reparse를 시험하고 권한 거절은 미검증으로 남긴다.
- 숨긴 native CLI/private desktop, 합성 fixture만 사용한다. GNU 결과를 MSVC/macOS 검증으로 표현하지 않는다.

## 검증과 handoff

[실제 Windows 검증](../validation/windows-file-identity.md)에 API 근거와 실제 명령/로그/source hash를 남겼다. Windows GNU에서 helper5harness 중4실제 검증, symlink1개는권한1314로미검증이다. hardlink/rename/pinned handle/replacement,512단위 초과 path버퍼, regular/type/containment, zero-ID 거절과 u64/16byte serde를 확인했다.

주 담당의 공유 wrapper/library/publication/legacy 연결을 포함한 최신 native fmt/clippy/test/release4개가 동일 source snapshot으로 통과했다. Windows library rename rowid 유지, hardlink ambiguity/replacement와 다중/legacy 공개 복구를 실제 검증했다. 초기 row_id cfg 및 공통 metrics dead-code 실패는 담당이 수정했으며 실패 로그도 보존한다.

새 dependency/feature가 없다. 강한 ancestor TOCTOU, symlink 실제 성공 경로/OneDrive reparse, MSVC/macOS/GUI/다른 Windows/FS는 남는다. 정상 Git 저장소가 없어 기존 .git을 보존한다. 중앙 ADR/domain/P0 게이트 갱신은 주 담당에게 전달했다.
