# Windows native CLI 환경과 검증

2026-10-01 WSL2 Ubuntu-22.04에서 실제 Windows x64 host의 Win32 프로그램을 실행했다. `cmd.exe /d /c ver` 결과는 10.0.26200.9457이다. WSL Linux 통과와 실제 Windows GNU 통과를 구분한다. Git 저장소/remote가 없어 remote Actions는 실행하지 않았다.

## 읽기 전용 조사와 권한

기본 sandbox의 Windows interop는 `WSL UtilBindVsockAnyPort: socket failed 1`(exit 1)로 실패했다. 사용자 승인된 read-only CLI를 require_escalated로 실행하니 실제 Windows 접근에 성공했다. 기존 cargo/rustc/rustup/MSVC/clang/cmake/ninja PATH 및 user .rustup/.cargo/bin, 기본 VS/SDK/LLVM/MSYS2 경로를 조사했으며 도구를 찾지 못했다. 기존 사용자 설정/데이터는 변경하지 않았다.

초기 fixed read-only 환경 PowerShell probe는 process-only ExecutionPolicy Bypass로 실행했으며 영구 정책은 변경하지 않았다. 이후 기본 정책으로 실행한 private-desktop PS1 probe는 Restricted/UnauthorizedAccess로 거절됐고 해당 script는 실행되지 않았다. 보안 정책을 변경하지 않았다. 이후 설치와 시험은 일반 native .exe CLI 및 프로젝트 Python/C# harness를 사용했다. PowerShell 실행 권한을 변경하는 절차는 제공하지 않는다.

## 프로젝트 전용 도구

주 담당이 설치를 승인했다. 최초 예산은 다운로드 300~600MiB, 도구 1~2GiB+build 몇 GiB, 20~40분 추정이었다. `.local-tools/windows` 안에 공식 고정 Rust 1.94.0 x86_64-pc-windows-gnu components와 LLVM-MinGW 20260922 msvcrt x64를 설치했다. 시스템 PATH/registry/사용자 rustup/cargo는 바꾸지 않았다. native process에 한정해 tool PATH, RUSTC/CARGO/RUSTFMT/RUSTDOC, CARGO_HOME/target/temp를 프로젝트 전용 경로로 지정한다.

`scripts/windows_tools_setup.py`는 [Rust 공식 manifest](https://static.rust-lang.org/dist/channel-rust-1.94.0.toml)의 SHA256을 확인하고 각각의 xz_hash를 검증한다. LLVM은 [공식 고정 release](https://github.com/mstorsjo/llvm-mingw/releases/tag/20260922)의 GitHub asset digest `1e936a4a694fc27f9625e3311f5ec5d6d99abfeaa514fd41c52a4f8347145d1a`와 173053724바이트를 확인한다. 정확한 URL/hash/size manifest는 `.local-tools/windows/tool-evidence.json`에 있다. 파일 확장과 archive 경로 탈출/link 정책을 검사한다. 프로젝트 소유 도구에 한정하며 외부 기존 설치를 제거하지 않는다.

| 구성 | 다운로드 bytes |
|---|---:|
| rustc 1.94.0 | 98004920 |
| cargo 1.94.0 | 11799452 |
| rust-std 1.94.0 | 26543528 |
| rustfmt 1.94.0 | 2944056 |
| clippy 1.94.0 | 5108936 |
| rust-mingw 1.94.0 | 5472212 |
| LLVM-MinGW 20260922 | 173053724 |
| 합계 | 322926828 |

`du -sb` apparent bytes: Rust 874640494, LLVM-MinGW 688694729, package license metadata 426897, archives+Rust manifest 323775956. 합계 1887538076이며 별도 cargo registry/build/log/temp는 포함하지 않는다. 이 도구는 개발 검증 비용이며 제품 최소 설치 크기가 아니다. Rust의 MIT/Apache notices와 std third-party license 파일(GPL/GCC runtime exception 포함), LLVM/MinGW의 동봉 LICENSE.TXT와 share 자료를 보존했다. 일괄 단일 라이선스로 재배포를 주장하지 않으며 MSVC/제품 패키징 license review는 별도 게이트다.

## private CLI 수명과 사용자 화면

기존 Windows .NET Framework `csc.exe`로 `scripts/windows_cli_host.cs`를 프로젝트 전용 exe로 컴파일했다. [CreateDesktopW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createdesktopw)로 고유 private desktop을 만들고 [CreateProcessW](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw)의 CREATE_NO_WINDOW|CREATE_SUSPENDED로 CLI를 시작한다. 소유 Job Object에 kill-on-close를 설정하고 **assign 성공 후** ResumeThread한다. 관리자 권한/Win32 desktop 보안 descriptor 변경/SwitchDesktop/SendInput은 사용하지 않는다.

합성 `windows_cli_fixture.cs`를 같은 기존 csc로 컴파일하고 private host에서 실행했다. 내부 1초 timeout, 명시 취소, controller stdin EOF 세 경로에서 소유 nested child가 종료됐으며 무관한 별도 fixture는 살아 있었다. 검증 controller 연결은 child에 상속하지 않고 stdin을 NUL로 분리한다. Python 검증 runner가 중단되면 native host에 취소를 전달하며 EOF에서도 소유 Job을 종료한다. PID가 재사용되어도 오판하지 않도록 child Process handle을 먼저 잡았다. outer test는 최종 15초 제한으로 실행하고 own fixtures를 모두 종료했다. 실제 결과: `PASS: native private CLI timeout killed owned descendant; unrelated fixture survived`, `dlpad-input-desktop-unchanged=True`. 앱 GUI는 실행하지 않았다. 이 helper는 시험용 수명 관리이며 제품 Windows process adapter 검증을 대신하지 않는다.

## 실행 명령과 현재 결과

```sh
python3 scripts/windows_tools_setup.py
/mnt/c/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe /nologo /target:exe '/out:D:\Project\dlpad\.local-tools\windows\windows-cli-host.exe' 'D:\Project\dlpad\scripts\windows_cli_host.cs'
/mnt/c/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe /nologo /target:exe '/out:D:\Project\dlpad\.local-tools\windows\windows-cli-fixture.exe' 'D:\Project\dlpad\scripts\windows_cli_fixture.cs'
.local-tools/windows/windows-cli-host.exe 15 'D:\Project\dlpad\.local-tools\windows\windows-cli-fixture.exe' --test 'D:\Project\dlpad\.local-tools\windows\windows-cli-host.exe' 
python3 scripts/windows_native_probe.py versions
python3 scripts/windows_native_probe.py fmt
python3 scripts/windows_native_probe.py check
python3 scripts/windows_native_probe.py test
```

native versions: rustc 1.94.0 (4a4ef493e, host x86_64-pc-windows-gnu), cargo 1.94.0. fmt 성공 0.68초. workspace/all-target/check 성공 24.54초. 최초 buildscript link는 rust-mingw 구성요소 누락 및 GCC runtime library 탐색 오류로 실패했다. 공식 rust-mingw와 process-only `RUSTFLAGS=-C link-self-contained=yes`로 해결했다. 최초 test는 dlltool PATH 누락으로 실패했고 private host process-only PATH에 프로젝트 도구를 추가해 해결했다.

새 vendor/lock 기반 실제 시험: app 12개와 core 20개 통과, engine 34개 통과/11개 실패, 전체 exit 101. [초기 실패 로그](windows-test-initial.log)를 보존했다. Windows canonical `\\?\D:\...` 경로의 Prefix만 쌓인 `\\?\D:`에서 symlink_metadata를 조회한 InstanceGuard/ToolManager 오류였다. 주 담당/해당 담당에게 Prefix push 후 조회를 건너뛰도록 수정 요청했다. source 수정은 이 작업 담당이 하지 않았다. 경고도 보고했다. Prefix 수정 뒤 1개 남은 InstanceGuard 중복 오류는 fs2의 Windows contention raw ERROR_LOCK_VIOLATION=33을 WouldBlock로만 분류한 것이었다. 주 담당이 raw contention 매핑과 Message::Managed의 Windows large enum boxing을 수정했다.

수정본 actual native fmt 1.23초/clippy 2.55초/test 7.16초 모두 exit 0. 시험은 app12/core20/engine47/diagnostic_cache3 총82개 통과했다. guardian/local_media는 Unix 전용이라 Windows 0개이며 FFmpeg 실제 옵션 시험은 환경변수를 지정하지 않아 실행하지 않았다. vendored iced_winit의 try_next deprecation 경고2개가 dependency에서 남았지만 workspace clippy exit은0이다. 각 operation의 source/manifest/lock before/after 해시는 동일했고 `.local-tools/windows/logs/*-inputs.json`에 보존했다.

실제 release compile도 45.85초(전체50.98초)에 exit0, dlpad.exe 6723584바이트와 headless helper264704바이트를 생성했다. 다만 작업 중 staging_recovery.rs 수정이 들어와 runner가 source snapshot 변경을 감지하고 exit3으로 stable final gate를 보류했다. 담당의 마지막 수정 이후 공통 명령을 재실행했다. 최종 동일 source snapshot에서 fmt/clippy/test/release 모두 exit0, 약1.2/2.2/5.7/18.5초였다. test82개 통과 및 release 앱6723584바이트의 SHA256은 c0a4234cb4844f50aad84ac2ab2094c7a1de7757a8b19be768ad47d6c59341a3다. [최종 JSON](windows-cli-final.json), [fmt](windows-fmt.log), [clippy](windows-clippy.log), [test](windows-test.log), [release](windows-build.log)를 보존했다. 각 네 operation의 before/after 입력뿐 아니라 operation 사이 입력도 모두 동일함을 별도 비교했다. lock SHA256은 ea848d8fee7be3aa0e2be1349399e89c46845bee4090df1ad14eee497cb5f71d이다. source가 바뀐 동안의 최초 성공을 최종 통합 성공으로 표시하지 않았다.

## 남은 OS 게이트

Windows GNU는 초기 목표 Windows MSVC 릴리스 검증을 대체하지 않는다. [Rustup Windows 안내](https://rust-lang.github.io/rustup/installation/windows.html)는 MSVC linker/library를 위한 Visual Studio 설치가 필요하다고 설명한다. 관리자 시스템 변경이 필요한 VS Build Tools 설치는 수행하지 않았다. MSVC 자체가 반드시 관리자 설치만으로 가능한 것은 아니다. [xwin primary README](https://github.com/Jake-Shadle/xwin)는 project-local Microsoft CRT/SDK 및 clang/LLVM 준비 경로를 설명한다. 고정 manifest/SDK/CRT/license/hash와 실제 native 실행을 별도 실험해야 하며 이번에 수행하지 않았다.

private desktop 생성/CLI 실행은 가능하지만 renderer/app GUI/한글 IME/TSF/200%/first-frame theme/스크린리더/트레이/설치·제거 검증은 아직 아니다. [Windows desktop 문서](https://learn.microsoft.com/en-us/windows/win32/winstation/desktops)에 따르면 visible/input desktop은 하나다. private desktop을 화면으로 전환하지 않고 실제 사용자 입력을 SendInput으로 주입하지 않는다. IME 및 screen reader는 별도 VM 또는 독립 시험 세션의 실제 시스템 구성으로 검증해야 하며, 글리프나 core 단위 시험으로 대신하지 않는다. macOS host는 없으므로 macOS 성공을 보고하지 않는다.
