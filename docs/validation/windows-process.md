# Windows native process 소유권 검증

2026-10-01 Windows 11 10.0.26200.9457 x64에서 프로젝트 전용 Rust 1.94.0 Windows GNU 도구로 제품 engine adapter를 실제 실행했다. GNU 검증이며 MSVC, macOS, Windows GUI/IME/a11y 검증을 대신하지 않는다.

## 구현 계약

`std::process::Child`는 주 스레드 핸들을 노출하지 않는다. Windows 전용 `CreateProcessW` adapter는 다음 순서를 사용한다.

1. non-inheritable `KILL_ON_JOB_CLOSE` Job 및 stdout/stderr pipe를 소유한다.
2. `STARTUPINFOEXW`의 stdio handle allowlist와 `PROC_THREAD_ATTRIBUTE_JOB_LIST`를 설정한다. Job 할당은 process 생성 자체에 적용한다. suspended 생성 후 별도 할당 사이에 부모가 crash하여 suspended orphan이 남는 구간도 제거한다.
3. `CREATE_SUSPENDED | CREATE_NO_WINDOW`로 생성하고 반환된 직접 process/thread handle을 소유한다.
4. `IsProcessInJob`으로 membership을 확인한 뒤 `ResumeThread`를 호출한다. Job 기능을 지원하지 않거나 할당이 실패하면 실행으로 fallback하지 않는다.
5. 취소/timeout/정상 root 종료 후 Job을 종료하여 pipe를 유지하는 자손도 정리한다. 부모 crash 시 non-inherited Job의 마지막 handle이 닫혀 자손이 종료한다. 오류 경로의 child Drop도 직접 소유 process handle만 종료하고 기다린다.

argv는 shell 없이 UTF-16 Windows quoting으로 전달한다. NUL 및 Windows command-line 길이 초과를 거절한다. 문자열은 32,767 UTF-16 단위 범위, attribute 저장 공간은 64 KiB 상한이다. 추가 product runtime/tool 의존성 없이 기존 windows-sys에 `Win32_System_Pipes` binding feature만 추가했다. 기존 작업당 Job/pipe에 두 startup attribute와 primary thread handle의 짧은 생존 비용을 더한다. 성능 benchmark를 실행한 것으로 표현하지 않는다.

## 실제 결과

- `python3 scripts/windows_native_probe.py test`: exit 0, 9.90초. 실행 전/후 source snapshot 동일. app13/core21/engine49/diagnostic3/Windows harness6 통과. [전체 로그](windows-process-test.log), [입력 snapshot](windows-process-test-inputs.json).
- Windows harness6 중4는 fixture 진입 함수이며 독립 기능 시험2개가 실제 동작을 시험한다. 별도 engine unit2개가 suspended 상태와 잘못된 Job 지정의 실패를 시험한다.
- 즉시 leaf를 생성한 root에 대해 정상 root 종료, 명시적 취소, 3초 timeout, worker 직접 강제 종료4case에서 root/leaf가 종료했다. 종료 전에 실제 process handle을 열어 identity를 고정하고 기다렸다. PID를 다시 찾아 종료하지 않았다. 동시에 실행한 무관한 fixture는4case 모두 생존했다.
- Resume 전 Job handle을 닫았을 때 suspended child가3초 이내 종료하고 stdout은 비어 있었다. Windows kill-on-close의 exit code가0인 실제 결과가 있어 nonzero exit code를 요구하지 않는다.
- invalid Job 지정은 실패했고 child 실행으로 fallback하지 않았다.
- 한글/공백 executable 이름, 빈 argv, 한글/emoji, quote, trailing backslash, shell metachar와 `%PATH%`를 원래 argv와 비교하여 literal 전달을 확인했다.
- 존재하지 않는 executable로 Job/pipe 생성 이후 실패를16회 warm-up하고64회 반복했다. 별도 `--nocapture` 관찰에서 handle 수139→139였다. 최초 측정에서는 일회성8 handle 증가를 관찰했다. Windows/harness 초기화와 반복 누수를 구분해 측정했으며 초기 증가의 정확한 귀속은 계측하지 않았다.
- 숨긴 전용 `windows-cli-host.exe`의 private desktop에서 실행했고 `dlpad-input-desktop-unchanged=True`였다. `SwitchDesktop`, `SendInput`, 제품 GUI 실행은 하지 않았다.
- Linux `cargo test -p dlpad-engine --lib process:: --locked`:12통과. `cargo test -p dlpad-engine --test guardian --locked`:8통과, guardian 부모 crash/자손/무관한 fixture 회귀 포함.

## 최신 통합 snapshot

주 담당의 source freeze 이후 `python3 scripts/windows_native_probe.py`의 `fmt`, `clippy`, `test`, `build`를 순차 재실행했다. 네 명령 모두 exit0이고 각각1.17/1.29/6.74/28.90초였다. **네 실행 전후와 실행 사이 전체 source hash가 동일**함을 별도로 assertion했다. [전체 입력 및 명령/시간/산출물 hash](windows-process-final.json), [최신 테스트 로그](windows-process-final-test.log), [최신 clippy 로그](windows-process-final-clippy.log), [최신 release 로그](windows-process-final-build.log), [최신 fmt 로그](windows-process-final-fmt.log).

최신 Windows app13/core21/engine49/diagnostic3/harness6 통과다. app의 Unix 전용 `measured_speed_and_eta_are_hidden_when_job_finishes`1개는 Windows에서 제외되므로 Linux app14와 구분한다. Linux guardian 및 media integration은 Windows에서0개이며 통과한 native OS 시험으로 계산하지 않는다.

GNU release `dlpad.exe`는6,735,360bytes, SHA256 `080d7dd4712b960f9383397593b911211ec17ac8c9f14f03c5f3654f8fc5668c`다. 공개 배포하지 않았고 제품 GUI를 실행하지 않았다. 네 CLI 실행 모두 input desktop 불변을 확인했다. Linux engine all-target clippy 및 담당파일 rustfmt도 통과했다. 이 증거는 JSON에 보존된 snapshot에만 적용하며 이후 새 소스 변경은 통합 담당자가 다시 검증한다.

## 지원 버전과 남은 게이트

`PROC_THREAD_ATTRIBUTE_JOB_LIST` API는 Windows 10 및 Server 2016부터 지원한다. [Microsoft 공식 API 계약](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute). 실제 시험은 위 Windows 11 host 하나에서 수행했다. **제품 최소 Windows 지원 버전을 Windows 10+로 정하는 것은 Proposed**이며 API 최소 버전을 실제 모든 Windows 10 버전 검증으로 바꾸어 표현하지 않는다.

MSVC 배포 빌드, Windows 10 실제 실행, 서명·설치 시험, Windows GUI/IME/a11y와 OS logout/sleep 시험은 남는다. 이 작업은 제품 Windows spawn→ownership 경쟁 조건을 해결하지만 전체 S03/최초 릴리스 완료를 뜻하지 않는다. 중앙 ADR 및 전체 P0 게이트는 통합 담당자가 갱신한다.
