<!-- Historical v16 snapshot. Current candidate is linked by README. -->
# Windows UI 검수 후보 — 2026-10-13

v16 배포 검수 후보와 진행 중인 entity-action identity 수정 소스를 구분한다. 아래 수치는 검증된 v16에 한정하며 후속 소스·빌드에 소급하지 않는다. 전체 P0·전체 물리 UI/UX·정식 릴리스는 미완료다.

1. [v16 설치 ZIP](../validation/evidence/dlpad-install-ui-cross-surface-v16-20261013.zip)을 새 폴더에 전부 압축 해제한다. payload를 함께 유지한다.
2. dlpad-install.exe에서 기본 설치 위치를 쓰거나 바꾼 뒤 **설치**를 누른다. 기존 폴더를 덮어쓰지 않는다.
3. **설치 확인 완료**에서 **앱 실행**을 누른다. 앱이 시작되면 설치 창은 닫히고, 앱 종료까지 감독 프로세스가 수명을 관리한다. 설치된 앱은 선택한 폴더의 registered/dlpad.exe다.

[설치 없이 실행할 v16 앱](../validation/evidence/ui-test-app-filled-focus-20261013/dlpad.exe)도 있다. 기본 저장 폴더와 폴더 선택을 제공하고 사용자 값은 커스텀할 수 있다. Windows NIC 관측·도구 설치·미리보기/범위 선택·고급 옵션 설명과 실행 계획·S10 P0 진단을 포함한다. 서명되지 않은 개발 후보이며 정식 배포 완료는 아니다.

ZIP16,726,709B, SHA256 `c52b52449438569a1a5300ea3cf49a3670c1061dcbbec5dcdef68bddd9c88e00`. v16 Windows1,309/0/33·43harness·source656은 원fmt/clippy/test+동일소스 release 복구 기록이며 단일 연속 실행이라고 주장하지 않는다. 63화면 팀검수, 정상·변조 설치 outer0/Job0·원파일/기록·27ID, Dark200 하단 명령 다섯 개 직접 픽셀을 확인했다. [현재 검수표](../validation/UI-UX-AUDIT-CURRENT-20261013.md), [남은 작업](../validation/P0-remaining-20261011.md).

v13 로컬 CI/성능/긴 경로는 과거별도 증거다. 긴 경로 앱 실행은 미검증이며 strict18의17통과/1실패와 v14 CI외부143 미완료를 보존한다. 물리 IME/reader/OS DPI·clean OS·upgrade/서명·전체 UI/UX는 남아 있다. Gitexit128·기존.git 보존 상태에서 hosted CI와 updater는 후속이다. 사용자는 아래 새 TEMP 프로필로 기존 Documents/DB와 분리해 직접 UI를 평가할 수 있다. 자동 GUI 검증은 계속 private desktop만 사용하며 사용자 화면으로 fallback하지 않는다.

## 도구 준비

새 후보의 도구 화면에서 yt-dlp, FFmpeg/ffprobe, Deno를 설치할 수 있다. yt-dlp direct 설치는 실제 버전 검증과 활성화를 함께 수행한다. FFmpeg와 Deno는 설치 후 해당 버전을 선택하고 **이 버전 활성화 → 다운로드에 이 관리 도구 사용**을 누른다. 이후 재실행하면 빈 실행 파일 경로에만 기존 활성 선택을 복원한다. 직접 지정한 외부 경로는 보존한다.

개발 설치 화면에는 세 도구의 선택 설치 체크박스가 있다. 기본은 모두 해제이며 앱 설치 성공 뒤 체크한 도구만 다운로드한다. 앱 설치 결과와 도구 공급 결과는 따로 표시된다. 도구 공급 실패가 이미 성공한 앱 설치를 실패로 바꾸지는 않는다. 설치된 archive 도구는 앱에서 선택·사용해야 한다.

설치 프로그램의 `--tool-data`는 별도 개발 시험 프로필용 절대 경로다. 위 PowerShell의 `$data`와 같은 경로를 지정해야 그 앱 시험 프로필에서 설치한 도구를 찾는다. 일반 실행은 기본 앱 data root를 공유한다. 이 기능 추가는 개발 설치 화면을 정식 서명 설치 프로그램으로 바꾸었다는 뜻은 아니다.

## PowerShell 실행 예시

`$exe`는 전달받은 개발 ZIP에서 직접 풀어 둔 실제 `dlpad.exe` 절대 경로로 바꾼다. 폴더는 복사·쓰기 가능한 사용자 TEMP 아래 새 GUID로 분리한다. 설치 GUI/로컬 CLI를 먼저 실행할 필요는 없다.

```powershell
$exe = 'D:\내 시험 폴더\dlpad.exe'
$preview = Join-Path ([System.IO.Path]::GetTempPath()) ('dlpad-user-preview-' + [guid]::NewGuid().ToString('N'))
$data = Join-Path $preview 'data'
$media = Join-Path $preview 'media'
$documents = Join-Path $preview 'documents'
$tmp = Join-Path $preview 'tmp'
foreach ($path in @($preview, $data, $media, $documents, $tmp)) {
    New-Item -ItemType Directory -Path $path -ErrorAction Stop | Out-Null
}
$start = New-Object System.Diagnostics.ProcessStartInfo
$start.FileName = $exe
$start.UseShellExecute = $false
$start.WorkingDirectory = Split-Path -Parent $exe
$start.Arguments = ''  # 기본 화면. 설정부터 보려면 '--settings'
$start.EnvironmentVariables['DLPAD_TEST_DATA'] = $data
$start.EnvironmentVariables['DLPAD_LIBRARY_ROOT'] = $media
$start.EnvironmentVariables['DLPAD_TEST_DOCUMENTS'] = $documents
$start.EnvironmentVariables['TEMP'] = $tmp
$start.EnvironmentVariables['TMP'] = $tmp
$app = [System.Diagnostics.Process]::Start($start)
$preview
```

환경값은 이 child와 그 자식에만 적용한다. 기존 사용자 환경 설정·실제 Documents·기존 앱 DB를 바꾸는 명령이 아니다. 시험 폴더는 자동 삭제하지 않는다. 창을 닫고 작업 종료를 확인한 후 경로를 기록해 둔다.

설정을 저장하면 `$data\theme.sqlite`(테마/저장 위치), 목록을 사용하면 `$data\library.sqlite`, 작업을 시작하면 `$data\jobs.sqlite`가 생길 수 있다. 읽기 전용 첫 시작에서 모든 DB 생성은 보장하지 않는다. 라이브러리 시작 경로는 `$media`이며 설정에서 기본 저장 위치를 눌러도 override된 `$documents\dlpad`가 후보가 된다. data와 media는 서로 포함되지 않는 형제 폴더여야 한다. 저장 위치를 직접 다른 실제 경로로 바꾸면 이 격리 범위에서 벗어난다.

## 5분 확인

1. 0–1분: 기본 화면을 열고 한글, 버튼 간격, 대비를 본다. 창을 줄였다 늘려 잘림·텍스트 소실을 확인한다.
2. 1–2분: Tab/Shift+Tab으로 이동해 현재 위치 표시를 보고, 설정에서 Light/Dark/System을 바꿔 읽기 편한지 확인한다. 한글 입력 필드에서 실제 키보드 입력·지우기·커서 이동을 직접 시험한다.
3. 2–3분: 파일 목록 화면에서 현재 경로가 위 시험 폴더인지 확인한다. 비어 있는 상태의 안내와 새로고침·다음 버튼 상태를 살핀다. 실제 영상이나 자격 증명을 넣을 필요는 없다.
4. 3–4분: 입력/옵션/도구 안내를 살핀다. 도구 선택이나 다운로드를 실행하려면 별도의 원본·출력 범위를 먼저 정한다. 첫 UI 평가는 다운로드 없이 가능하다.
5. 4–5분: 창을 닫고 종료를 확인한 뒤 같은 `$start`로 다시 실행한다. 테마 선택과 시험 경로 복원을 확인한다. 불편한 점은 화면 이름·동작 순서·기대한 결과·실제 결과로 적는다.

## 확인한 source 계약 / 한계

- `accessible_cpu_main.rs`의 data override와 InstanceGuard는 새 `$data`를 사용한다. 다른 실행의 기존 data와 혼합하지 않는다.
- `accessible.rs` 초기 output은 DLPAD_LIBRARY_ROOT, 설정 초기 표시는 `--settings`다.
- `accessible_library.rs` 명시 Default 경로는 DLPAD_TEST_DOCUMENTS가 있으면 그 아래 dlpad로 해결한다. engine 기본 경로는 OS Documents/dlpad지만 위 override가 이를 피한다.
- 입력·프로세스·도구·네트워크의 모든 행동을 이 안내가 sandbox로 강제하지는 않는다. 사용자가 다른 경로를 선택하거나 도구/다운로드를 실행하면 해당 행동은 별도로 검토해야 한다.
- 실제 실행·사용자 물리 IME·reader·DPI 검증은 이 문서 작성에서 수행하지 않았다.
