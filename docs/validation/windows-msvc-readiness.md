# Windows MSVC 준비 상태 조사

2026-10-03 UTC12:33 native Windows 읽기 전용 관측. [작업 계약](../work-items/S11-windows-msvc-readiness.md), [실제 결과](evidence/windows-msvc-readiness-20261003/result.json), [native Rust 버전](evidence/windows-msvc-readiness-20261003/native-versions.json), [공식 자료 목록](evidence/windows-msvc-readiness-20261003/official-sources.json).

## 실제 준비 상태

기존 windows-cli-host.exe의 private desktop에서 전용 query.ps1을 실행했다(exit0). native Windows version10.0.26200.0/x64이며 host가 private-desktop=true/input-desktop-unchanged=True를 기록했다. 사용자 입력·clipboard·GUI·설치·환경 설정은 변경하지 않았다. 단발 프로세스의 PowerShell ExecutionPolicy Bypass 인자는 기존 machine/user policy를 저장하거나 수정하지 않는다. 실제 argv와 query/host SHA는 result.json에 있다.

| prerequisite | 실제 관측 |
|---|---|
| PATH cl.exe/link.exe/clang-cl.exe | 모두 없음 |
| 알려진 Visual Studio Installer/vswhere.exe | 없음 |
| 알려진 VS2019/2022/18/2026, BuildTools/Community/Professional/Enterprise 경로 | 존재하는 설치 없음 |
| Program Files/Program Files(x86) Windows Kits10/11 | 존재하는 SDK 없음 |
| 프로젝트 native Windows rustc/cargo | 존재, rustc1.94.0/Cargo1.94.0 |
| native compiler host | x86_64-pc-windows-gnu |
| 프로젝트 x86_64-pc-windows-gnu std | 존재 |
| 프로젝트 x86_64/aarch64-pc-windows-msvc std | 없음 |

조회는 알려진 경로와 PATH, 설치당 version 최대16개로 제한했다. custom 설치 폴더·다른 사용자 데이터·전체 드라이브를 검색하지 않았다. 따라서 머신에 숨겨진 모든 custom MSVC 설치가 없다는 증명은 아니며, **현재 발견한 구성으로 MSVC build 준비가 안 됐다**는 판정이다. 기존 GNU의 실제 테스트/release 결과는 MSVC 결과로 재사용할 수 없다.

## 필요 도구와 사용자 전용 경로 판단

Rust 공식 지원 문서는 Windows MSVC target과 Visual Studio host tooling을 전제로 하며 비Windows host에서 MSVC cross compilation을 supported 경로로 보장하지 않는다. WSL GNU 통과만으로 native MSVC를 증명하지 않는다. [Rust target 문서](https://doc.rust-lang.org/stable/rustc/platform-support/windows-msvc.html).

공식 일반 경로는 Visual Studio Build Tools의 x64/x86 C++ compiler component와 Windows SDK다. 정확 설치 뒤 cl.exe/link.exe, MSVC runtime/import libs, SDK headers/UM/UCRT libraries, 선택한 Rust1.94 MSVC std를 확인하고 compile/test해야 한다. 프로젝트는 rusqlite bundled C compilation도 있으므로 Rust std만 추가해서 준비 완료로 판단하지 않는다. [Microsoft 설치/컴포넌트 안내](https://learn.microsoft.com/en-us/cpp/overview/acquire-msvc?view=msvc-170).

Microsoft는 초기 설치 및 MSVC command-line 설치에 관리자 권한을 요구한다. D: 또는 프로젝트 폴더를 installPath로 고르더라도 일부 SDK/tools는 시스템 drive에 설치될 수 있다. 따라서 사용자가 허용한 사용자/프로젝트 전용 도구 설치 범위를 이유로 이 installer를 실행하거나 UAC를 승인하지 않았다. 일반 Build Tools 설치는 사용자 지시의 관리자 시스템 변경 별도 승인 범위에 해당하므로 그 동작만 보류한다. [초기 설치 권한](https://learn.microsoft.com/en-us/visualstudio/install/install-visual-studio?view=visualstudio), [설치 위치의 한계](https://learn.microsoft.com/en-us/visualstudio/install/change-installation-locations?view=visualstudio).

공식 Windows SDK는 NuGet 공급도 제공한다. 이는 사용자/프로젝트 디렉터리에서 SDK 부품을 취득·검증할 수 있는 후속 조사 후보이며, 전체 MSVC compiler/linker/runtime이 들어 있는 portable Build Tools 대체물이라고 확인한 것은 아니다. 고정 예시10.0.26100.4948의 Microsoft.Windows.SDK.CPP는149.3MB, 같은 버전 x64 package는49.73MB로 게시돼 있다. 압축 download 합계199.03MB이며 추출 용량·완전한 compiler/VC runtime 구성·프로젝트 빌드 가능성은 미측정이다. 최신판 선정이나 설치는 하지 않았다. [공식 SDK 다운로드](https://learn.microsoft.com/en-us/windows/apps/windows-sdk/downloads), [공통 SDK](https://www.nuget.org/packages/Microsoft.Windows.SDK.CPP/10.0.26100.4948), [x64 SDK](https://www.nuget.org/packages/Microsoft.Windows.SDK.CPP.x64/10.0.26100.4948).

이번 조사에서 **공식적으로 지원되고 라이선스/필수 파일이 확인된 완전한 user-only MSVC toolchain**은 찾지 못했다. 비공식 compiler harvesting/portable script를 추천하거나 실행하지 않았다. 공식 LLVM/clang-cl로 C compilation을 대체하더라도 MSVC target의 Windows SDK·runtime/link libraries 요구와 실제 native link 검증은 남는다. SDK NuGet만 받고 MSVC gate를 통과했다고 표시하지 않는다.

## 크기·런타임·라이선스와 남은 결정

Microsoft Build Tools2022의 공식 가용 disk 범위는 선택 feature에 따라2.3~60GB다. 이 숫자는 일반 requirements 범위이며 dlpad용 정확 선택 설치량 또는 download 크기가 아니다. MSVC/Rust/SDK는 개발 도구 비용이며 앱 번들 크기에 자동 합산하지 않는다. 현재 설치하지 않아 실제 추가 디스크/peak RSS/빌드 시간과 최종 C runtime 배포·static link 비용은 미측정이다. [공식 system requirements](https://learn.microsoft.com/en-us/visualstudio/releases/2022/system-requirements).

Build Tools 사용 권한과 재배포는 동봉 라이선스/선택된 Visual Studio 권리와 함께 확인해야 한다. Microsoft licensing whitepaper는 조건부 Build Tools 사용 범위를 설명하며 공개소스 의존성 예외를 무조건 모든 독립 프로젝트에 적용한다고 쓰지 않는다. 현재 사용자 계정·보유 Visual Studio license·공개 프로젝트 여부를 추측하지 않았다. SDK NuGet license link는 공식 aka.ms/WinSDKLicenseURL이지만 웹 도구가 binary content를 읽지 못해 전체 내용 미검수다. 실제 도구 채택 전 해당 정확 payload의 라이선스를 검수해야 한다. [Microsoft licensing whitepaper](https://visualstudio.microsoft.com/wp-content/uploads/2025/04/Visual-Studio-Licensing-Whitepaper-April2025.pdf).

후속 선택은 기존 적법한 MSVC/SDK custom 경로 제공, 별도 승인된 관리자 설치, 또는 공식 user-only 부품 경로의 추가 실험이다. root가 정확 toolchain/source snapshot과 private Windows CLI 환경을 조정한다. 현재 MSVC fmt/clippy/test/release, 제품 GUI·설치·서명·3 OS CI는 완료가 아니다. 정상 Git 저장소가 없어 기존 .git을 보존하며 remote CI도 실행하지 않았다.

## 실행 명령과 검증

기존 host를 argv로 실행해 PowerShell query를 -NoLogo/-NoProfile/-NonInteractive/-File로 실행했다. `windows-cli-host.exe 60 C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe ...` exact argv는 evidence에 있다. 추가 두 private CLI는 프로젝트 rustc.exe --version --verbose, cargo.exe --version이고 모두exit0/desktop unchanged=True다. 읽기·공식 문서 조회만 수행했으며 설치·package download·Rust target/Cargo/source 변경·프로세스 signal·사용자 화면 fallback은 없다. 준비 상태 조사만 완료했고 MSVC 인수 게이트는 남는다.
