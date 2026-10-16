# S04: Windows 관리 runtime 도구의 실제 비공개 GUI 인수

- 상태: in-progress
- 분류: test
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당: metrics_recovery
- 수정 파일: 신규 scripts/windows_runtime_tools_observer.cs, scripts/windows_runtime_tools_probe.py, 이 문서
- 의존성: 현재 Windows release CPU 바이너리/관리 UI 및 기존 native 비공개 desktop/Job/UIA harness

## 범위 및 인수

먼저 실제 privateDesktop UIA에서 yt-dlp/Deno/FFmpeg·ffprobe/yt-dlp 탭 전환, fresh 합성 cache의 빈 목록, runtime 공급 미채택 안내와 설치 disabled, WM_CLOSE·소유Jobempty·inputdesktop불변을 확인한다. 기존 Rust/Cargo/schema/probe는 수정하지 않는다. immutable binary 복사 SHA와 sourcebaseline을 기록하며 다운로드/clipboard/visible desktop fallback을 사용하지 않는다. 합성 managed version 실행파일 fixture를 필요로 하는 Select/Use 정확 field 전달 및 wrongversion prioractive 보존은 별도 실제 인수가 필요하며 첫 단계 성공으로 확대하지 않는다.

## 검증

코드 구현과 실제 native 실행 결과를 아래 기록한다. 현재 root full collector와 파일 소유권을 분리한다.

## 실제 결과와 수집기 교정

기본 빈 목록의 첫/두번째 시험은 Refresh 이전 inventory None 상태를 EMPTY로 기대하여 실패했다. 제품 동작을 바꾸지 않고 실제 기본 yt-dlp 탭의 목록 갱신 동작을 수집기에 추가했다. 세 번째 `python3 scripts/windows_runtime_tools_probe.py /mnt/d/project/dlpad/docs/validation/evidence/windows-runtime-tools-third`는 terminal exit 0/실제 GUI PASS: yt-dlp→Deno→FFmpeg·ffprobe→yt-dlp 왕복, runtime 설치 disabled와 공급 보류 안내, 빈 목록, WM_CLOSE, 소유 Job empty, private desktop close, input desktop 불변이다.

합성 C# native deno.exe/ffmpeg.exe/ffprobe.exe가 고정 1.0.0을 보고하도록 준비하고 원 1.0.0 active 및 2.0.0 inactive manifest를 생성했다. wrong-version 첫 실행은 Deno Use 필드에 올바른 경로가 전달됐으나 expected `D:\project`와 actual `\\?\D:\Project` 표기 차이로 수집기 비교가 실패했다. 실제 원 파일 핸들의 GetFinalPathNameByHandle로 기대값을 구해 literal exact 비교로 교정했다. [진단 실제 필드](../validation/evidence/windows-runtime-tools-seeded-diagnostic/gui/typed-path-1.json)를 보존했다.

`windows-runtime-tools-seeded-second`의 실제 native GUI는 PASS다. Deno/FFmpeg 각각 2.0.0의 버전 불일치 활성화 거절을 실제 관찰하고, 기존 활성 1.0.0의 Use가 정확한 다운로드 Deno/FFmpeg Edit 필드에 원 native canonical 경로를 전달했다. [실제 GUI 결과](../validation/evidence/windows-runtime-tools-seeded-second/gui/native-result.json)와 각 typed-path JSON/field PNG를 보존했다. 다운로드를 시작하지 않았다. WM_CLOSE/Job empty/private desktop close/input desktop 불변도 PASS다.

단, 전체 collector result는 false다. 실행 중 다른 담당의 신규 windows_managed_journal.rs가 추가돼 sourcebefore/after 항목 집합이 달라졌다. 기존 파일 SHA 변경은 없었지만 이를 source 불변 PASS로 바꾸지 않았다. immutable GUI exe는 동일하며 전역 최신 snapshot 검증은 root 모듈 통합 뒤 재실행한다. 현재 Rust/source collector 진행과 무관하게 수집기 후속 개선만 게시했다.

[독립 읽기 검증](../validation/evidence/windows-runtime-tools-seeded-second/independent-payload-active-review.json)에서 두 도구의 DB active는 여전히 1.0.0이고 각 native 실행파일/notice 크기와 SHA는 원 저장 manifest 값에 일치했다. 당시 메모리 안의 pre-run map은 aggregate false 경로에서 파일로 저장하지 않았으므로 이를 완전한 별도 전후 SHA snapshot으로 주장하지 않는다. 향후 collector는 seed-before/after와 observer source 복사본, foreign sentinel 전후 hash를 저장한다. 마지막 교정된 collector 버전의 실제 재실행과 sentinel 보존 증거는 아직 없으며, 전역 freeze 준비 뒤 root 인수로 남긴다. 정상 inactive 버전의 새 활성화 성공은 이번 GUI 사례에 포함하지 않았다.

실제 seeded GUI immutable 앱 SHA256: `79a1d38f99de59c5a942c71842e59339dbf847a0ad4b2beb8713da937d7bb747`. 신규 Python py_compile exit 0이며 프로젝트 Rust/Cargo/기존 probe 수정은 없다.
