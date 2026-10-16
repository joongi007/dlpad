# v25 이하 Windows 검수 안내 (고정 기록)

당시 설치 검수본은 **v25**였다. 앱 실행 파일은v24와같으며 삭제후재설치의기록선택을수정했다. 사용자 수동 검수는 나중에 진행하며, 아래 안내는 그때 사용할 경로와 흐름이다. 이전 설치본을 실행하면 최신 UI 변경을 볼 수 없으므로 압축 파일의 버전을 먼저 확인한다.

## 실행할 파일

1. [v25 설치 ZIP](../validation/evidence/dlpad-install-ui-feedback-v25-20261014.zip)을 새 폴더에 모두 푼다. `payload`를 함께 유지한다.
2. `dlpad-install.exe`에서 기본 설치 위치 또는 선택한 상위 폴더를 사용해 설치한다. 기존 설치 폴더는 덮어쓰지 않으며 설치 버튼이 비활성화된다. 기존 대상은 **다시 검증**으로 확인한다.
3. **설치 확인 완료**에서 **앱 실행**을 누른다. 설치된 파일은 선택한 폴더의 `registered/dlpad.exe`다.

설치 없이 보려면 [v25 portable ZIP](../validation/evidence/dlpad-user-preview-ui-feedback-v25-20261014.zip)을 풀고 `dlpad-cpu.exe`를 실행한다. 앱 자체가 기본 저장 폴더를 제공한다. 아래 TEMP 프로필 예시는 기존 앱 설정과 분리해 시험하고 싶을 때만 사용한다.

설치 ZIP은17,164,983바이트, SHA256은 `291d2d7a077b2c46494283d4470f34360c8e85296df3287ad45b46c6b4237996`이다. 서명되지 않은 개발 검수본이며 정식 배포가 아니다.

## 화면 흐름

- **다운로드**: URL 확인·영상 미리보기·한 개의 구간 축과 시간 입력·품질·설명 있는 고급 옵션을 사용한다. 영상 제목을 유지하는 파일명 옵션은 기본 제공하며 job ID로 동명을 구별한다.
- **파일 목록**: 현재 폴더는 읽기 전용으로 표시한다. 파일 선택 후 미리보기·경로 확인·기본 앱으로 열기·폴더 열기·새 이름으로 복사할 수 있다. 더블클릭이나 Enter는 앱 안 미리보기다. 복사는 원본을 바꾸지 않고 기존 대상도 덮어쓰지 않는다.
- **작업 이력**: 저장된 작업과 실패·복구 안내를 확인한다.
- **설정**: 일반·도구·저장 위치로 나뉜다. 기본 폴더는 저장 위치에서 바꾸며 이번 다운로드의 임시 저장 위치는 별도다. 화면을 바꿔도 복사 진행·중단 대기를 하단에 표시한다.

## 다운로드 도구

도구 목록은 캐시를 먼저 표시하고 갱신한다. 설치·선택·실행은 실제 상태를 다시 확인하므로 캐시만으로 시작하지 않는다. **다운로드에 이 관리 도구 사용**은 검증된 앱 관리 설치본을 다운로드 실행에 선택한다는 뜻이다. 직접 지정한 외부 경로는 보존한다.

yt-dlp 새 버전은 조회된 정확한 버전을 설치한 뒤 명시적으로 선택한다. FFmpeg·ffprobe와 Deno는 현재 승인된 고정 공급 버전 범위다. 설치 화면의 선택 설치는 기본 해제이며, 도구 설치 실패가 이미 완료한 앱 설치를 취소하지 않는다. 설치 프로그램의 `--tool-data`는 개발 시험용 별도 프로필이며 일반 실행은 기본 앱 data root를 공유한다.

## 실제 확인한 범위

v25/source671 전체4검사1,542/0/42와 실제 private 설치→제거→동일위치재설치→설치기를다시열어재검증을Root가독립확인했다. 기존기록은직접삭제하지않는다. 새기록은빈generation을선택하고현재설치가있는경우실제일치기록을조회한다. 기존앱폴더의업그레이드/덮어쓰기는제공하지않는다. [재설치검토](../validation/evidence/windows-msvc-ui-feedback-v25-reinstall-private-v3-20261014/root-review.json).

아래v24는변경전고정소스의기록이며새설치기전체UI/물리인수로확대하지않는다.

이전 v24 Windows 전체 네 명령1,538/0/42, 선택 정보6조합/24화면 직접 팀 검수, 정상 설치·577/598UTF-16 긴 경로 실행을 독립 대조했다. 변조 실행 거부·원파일 보존도 독립 대조했다. 정상·긴경로 설치88원화면을 직접 팀 검토했다. 최신 로컬CI15단계·8Cargo명령·7개 결과 수집도 통과했다. [현재 잔여 작업](../validation/P0-remaining-20261011.md).

이전 v24 합성 목록의 세60초 평균CPU는0.676534%·0.546358%·0.624538%이며 원카운터로 독립 재계산했다. 메모리18구간을 충족했고, 추가 strict10초 CPU진단3구간 실패를 보존한다. 이를 성능 전부 합격 또는 원인 해결로 표현하지 않는다. 이전 v22 앱73화면과 v23 worker계측은 각각 이전 소스의 기록이다. 물리 hover·IME·reader·OS DPI·clean OS 및 사용자 최종 UI/UX 인수는 별도다. 실제 설치 전환·되돌리기는 아직 지원하지 않는다. Git 조회exit128을 기록하고 기존 `.git`을 보존했으며 hosted CI와 앱 updater는 후속이다.

## PowerShell 실행 예시

`$exe`는 전달받은 개발 ZIP에서 직접 풀어 둔 실제 `dlpad.exe` 절대 경로로 바꾼다. 폴더는 복사·쓰기 가능한 사용자 TEMP 아래 새 GUID로 분리한다. 설치 GUI/로컬 CLI를 먼저 실행할 필요는 없다.

```powershell
$exe = 'D:\내 시험 폴더\dlpad-cpu.exe'
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
- private desktop 자동 실행 근거와 사용자 물리 IME·reader·DPI 인수는 구분한다. 사용자 물리 인수는 아직 수행하지 않았다.

