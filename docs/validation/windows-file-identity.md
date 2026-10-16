# Windows handle 기반 file identity 검증

2026-10-01~10-02 실제 Windows 11 10.0.26200.9457의 native Windows GNU Rust1.94.0에서 합성 임시 파일을 시험한다. 사용자 Documents·실제 미디어·시스템 설정·사용자 데스크톱을 사용하지 않는다.

## 계약과 비용

`from_file(&File)`는 직접 소유 handle의 `FILE_ID_INFO`를 읽어 `NativeFileIdentity::Windows { volume_serial: u64, file_id: [u8; 16] }`를 반환한다. 128bit ID를64bit로 줄이지 않으며 volume serial과 함께 비교한다. [Microsoft FILE_ID_INFO 계약](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_info).

128bit ID가 전부0이면 Unsupported로 거절한다. 일부 파일시스템이128bit ID를 지원하지 않을 때0을 반환하고 무시해야 한다는 [Microsoft FSCC 계약](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/e4185a8a-ed8d-4f98-ab55-ca34dc8916e6)을 따르는 실패안전 정책이다. volume serial0 자체는 금지하지 않는다. 실패한 identity를 None==None으로 같은 파일이라 추정하면 안 된다.

공유 enum/serde는 주 담당이 소유한다. JSON에서는 u64 serial과16개u8 배열로 보존한다. u64::MAX와 byte255까지 Rust serde 왕복을 검사한다. 라이브러리 string 및 publication plan 연결은 주 담당의 계약으로 관리한다. 이 관측치는 파일의 현재 정합성 근거이며 콘텐츠·출처의 영구 identity가 아니다. 삭제 뒤 ID 재사용과 같은 파일의 내용 변경 가능성은 남는다.

새 dependency/feature/runtime/tool 설치가 없다. 기존 windows-sys FileSystem binding과 kernel32 API를 사용한다. FILE_ID_INFO24byte, 속성8byte, 최종경로512 UTF-16 초기 버퍼를 사용하며 최종경로는32,768단위 상한이다. 파일 전체를 읽거나 해시하지 않는다. OS metadata I/O를 수행하므로 worker 호출 계약을 유지한다. 정밀 성능 benchmark는 수행하지 않았다.

## 경로와 reparse 증거

- `open_regular_no_reparse`는 절대경로의 기존 파일을 FILE_READ_ATTRIBUTES로만 연다. std Windows OpenOptions의 extended-length 정규화를 유지하고 final component에 OPEN_REPARSE_POINT를 적용한다. 디렉터리·reparse·비disk handle을 거절한다.
- `final_path`는 normalized DOS handle path를 bounded buffer로 읽는다. [공식 반환 길이 계약](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew)에 따라 작은 버퍼를 확장한다.
- `ensure_regular_under`는 root directory와 regular file의 직접 handle 속성·volume·최종경로를 비교한다. root가 reparse이거나 file 최종경로가 root의 strict descendant가 아니면 거절한다.
- from_file 자체에는 path containment 의미가 없다. OPEN_REPARSE_POINT는 ancestor를 고정하지 않는다. 최종경로 비교도 나중에 다시 연 pathname이 같은 파일이라는 영구 증거 또는 동시 ancestor 교체의 완전한 TOCTOU 방어가 아니다. main publication은 별도의 stored canonical-path/native-ID 비교를 수행한다.
- 알 수 없는 cloud/OneDrive reparse도 보수적으로 거절한다. 해당 저장 위치 지원과 stronger ancestor pinning은 별도 게이트다.

## 초기 실제 검증과 한계

`python3 scripts/windows_native_probe.py test`에서 Windows handle identity4unit 및 main Windows publication unit을 포함한 엔진54개가 통과했다. [초기 로그](windows-file-identity-test.log), [해당 source snapshot](windows-file-identity-test-inputs.json). 이 초기 source 이후 zero-ID 거절과 main library 연결이 추가되어 최신 증거는 후속 갱신한다.

- 같은 파일의 hardlink/rename은 동일 ID였다. 원래 이름에 같은 내용·크기의 새 파일을 만들면 ID가 달랐다. 기존 열린 handle은 교체 후에도 원래 ID를 유지했다.
- 한글 temp root와512 UTF-16 초과 경로에서 handle 최종경로 버퍼 확장과 root 포함을 확인했다. 이름 prefix만 비슷한 다른 directory의 파일은 거절했다.
- 디렉터리·pipe·relative·parent traversal·NUL path를 거절했다.
- main publication의 partial2artifact 공개 복구·반복 복구·matching staging 정리 및 same-size replacement 거절 시험이 native Windows에서 통과했다.
- **심볼릭 링크 시험은 미검증**이다. hidden CLI의 `--nocapture` 실제 실행에서 symlink 생성이 os error1314(권한 부족)였고 시험이 skip했다. 테스트 harness의 pass를 symlink 동작 검증으로 계산하지 않는다. 관리자 권한·Developer Mode·보안 정책 변경은 하지 않았다. native symlink와 directory-reparse 성공 경로는 별도 환경에서 필요하다.

초기 native clippy는 wrapper 연결 이후 통과했다. 최신 library rename test 연결 뒤 row_id Unix cfg의 native 컴파일 실패를 발견해 주 담당이 수정했다. [초기 실패 로그](windows-file-identity-initial-failure-clippy.log)를 보존했다. 후속 R01 ProcessingFeed의 Windows dead-code도 발견해 담당에게 전달했다. [metrics 실패 로그](windows-file-identity-metrics-clippy-failure.log). 이 두 실패를 성공으로 보고하지 않는다.

zero-ID 정책 및 최신 main library/publication/legacy 연결을 포함한 native 전체 `test`는 app13/core21/engine59/diagnostic3/harness6 통과,12.43초, 실행 전후 snapshot 동일이었다. [연결된 최신 테스트 로그](windows-file-identity-connected-test.log), [입력 snapshot](windows-file-identity-connected-test-inputs.json). Windows rename row ID 유지, hardlink ambiguity·replacement 실패안전, 최신다중 및 legacy 단일 공개 복구를 포함한다.

전용 helper를 hidden host에서 `file_identity::windows::tests --nocapture`로 별도 실행했다. [실제 helper 로그](windows-file-identity-helper.log)는5개 harness pass 중4개 실제 검증과 symlink1개 os1314 skip을 구분한다. main이 stderr observer를 모든 OS에서 호출하도록 연결한 뒤 최신 전체 `fmt/clippy/test/build`를 순차 재검증했다. 모두exit0, 각각1.41/3.56/12.13/21.75초였으며 **네 명령 전후와 명령 사이 전체 source hash가 동일**했다. [최종 증거](windows-file-identity-final.json), [fmt](windows-file-identity-final-fmt.log), [clippy](windows-file-identity-final-clippy.log), [test](windows-file-identity-final-test.log), [release](windows-file-identity-final-build.log).

최신 native app13/core21/engine59/diagnostic3/Windows harness6 통과다. processing-log opt-in1개는ignored이며 symlink1개는permission skip이므로 해당 실제 기능 검증으로 계산하지 않는다. Unix 전용 soak/guardian/local-media integration은 native Windows에서0개다. Windows 실제 FFmpeg provider 연결 실증을 이 정적 검사 통과로 대신하지 않는다. 소유 작업 외 source 변경은 통합 담당자가 다시 검증한다.

GUI/IME/a11y, MSVC, Windows10/다른 파일시스템, macOS 실제 host, 공개 설치/서명, path 경쟁 완전 방어는 완료로 주장하지 않는다. private CLI host는 사용자 input desktop 불변을 확인했고 제품 GUI는 실행하지 않았다.

최종 GNU `dlpad.exe`: 6781952bytes, SHA256 `d3b67c5b3774c6322e1f95b822e340d6466301fe35f4e06f1a9eac3be3c89976`. 공개 배포하지 않았다.
