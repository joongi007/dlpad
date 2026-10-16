# S04: Windows Deno 고정 공급 인수

- 상태: in-progress
- 분류: supply
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: diagnostic_cache; scripts/windows_deno_supply_probe.py, 본 문서 및 전용 새 evidence만
- 선행 작업: 공식 Deno v2.9.7 공급 후보, 관리 archive 및 검증 선택 API

## 사용자 결과

고정 Windows x64 공식 ZIP/checksum과 정확 source revision/license/third-party 자료를 수집한다. 원 archive와 실행 파일 SHA를 구분한다. 공급 채택·실제 Windows 활성화 성공은 자료 수집만으로 주장하지 않는다.

## 계약과 범위

공식 GitHub API/releases/raw/codeload만 HTTPS bounded download, 새 소유 evidence/create-only 출력. ZIP 단일 deno.exe 검증/스트림 추출. source archive는 저장만 하며 실행하지 않는다. Native 실행·Cargo·제품/공유 문서는 root 담당이다. 사용자 cache/credential/JS network/desktop 접근 없음.

## 인수 조건

공식 checksum과 고정 archive bytes/SHA 일치, 추출 executable bytes/SHA 및 CRC 관측. v2.9.7 commit full SHA와 license/Cargo.lock/third-party tree의 정확 revision 자료를 보존하고 미확보 자료를 누락으로 보고한다. trust/attestation와 transitive source 완비, 실제 native version/activation 비용은 독립 게이트다.

## 검증과 handoff

공식 release/설치 문서 primary browse 완료, 수집기 작성 중. 관측 날짜 2026-10-04. https://github.com/denoland/deno/releases/tag/v2.9.7 및 https://docs.deno.com/runtime/getting_started/installation/ .

## 실제 수집 결과 (2026-10-04)

`python3 -m py_compile scripts/windows_deno_supply_probe.py` exit0. 최종 `python3 scripts/windows_deno_supply_probe.py docs/validation/evidence/windows-deno-supply-root-header-fix` exit0, `result.json` status=`materials-collected-not-adopted`, native_executed=false/adopted=false. source/Rust/Cargo 및 기존 수집기는 변경하지 않았다.

- [공식 고정 ZIP](https://github.com/denoland/deno/releases/download/v2.9.7/deno-x86_64-pc-windows-msvc.zip): 42,630,221 B, SHA256 `a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238`. API asset digest·177 B 공식 checksum과 일치. 정확 ZIP inventory는 `deno.exe` 한 개, notice 없음.
- `deno.exe`: 97,462,048 B, SHA256 `e020f3e232bd16e33768dee528e5983349c962952051ced0a5d58ad42f5d9b33`, CRC32 1414633474. 실제 ZIP stream/CRC를 끝까지 읽었다. 실행·코드서명 검증은 하지 않았다.
- [정확 source revision](https://github.com/denoland/deno/commit/0c071246a412575e07423263404a5d13e7ed6aa2): `0c071246a412575e07423263404a5d13e7ed6aa2`. [공식 source archive](https://codeload.github.com/denoland/deno/tar.gz/0c071246a412575e07423263404a5d13e7ed6aa2) 32,789,135 B, SHA256 `fb58a70f221ea64e9b44ade6bc4656ab8427fc131f13700bcda2788a775816fb`; bounded sequential inventory 19,137개/declared90,653,033 B. extractall/실행 없음.
- [LICENSE.md](https://raw.githubusercontent.com/denoland/deno/0c071246a412575e07423263404a5d13e7ed6aa2/LICENSE.md) MIT, 1,074 B, SHA256 `f62497fffecc0852960c8d3e6934b9db86d16396e9b604072e923892cae3a588`. 같은 revision Cargo.lock 294,948 B/`4b501f05a28bf12c453593f0b558407890575b5f34b9dcbb2dc8e172d58a0c95`, Cargo.toml 21,077 B/`680e2ecaaef553b9329080897a8051af0ef83629b10d620451001e8591ae5e9e` 확보.
- 정확 source의 bundled Node type license 1,141 B/`c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383`, undici license 1,090 B/`a6db8096b2707bc0102d256917d4d33f298ba36d8c3f25de067a2b5bb379db27`, webgpu MIT 및 .gitmodules/compiler source를 따로 보존했다. root `third_party/` directory는 해당 revision에 없다. 이것을 third-party 의존성 없음으로 해석하지 않는다.
- Cargo.lock의 V8 crate는 150.4.0/checksum `42a978ff11f15b24e5c05a7123cf2b68f41e763546699781a924ef4e2cf43a49`다. Deno source snapshot은 외부 Cargo/V8/TypeScript 등 모든 대응 source·embedded notice를 포함했다는 증거가 아니다. 정확 transitive mapping/source 및 binary attestation 검증은 미완료다. GitHub가 commit verified/immutable release라고 표시한 것과 binary attestation 실제 검증은 구분한다.

## 비용과 제품 설치 결합 제안

현재 자료 기준 executable 영구 용량은 약92.95 MiB, 원 ZIP 약40.66 MiB, 설치 staging+최종 executable+원 archive의 동시상한 비용은 최소 약226.56 MiB에 notice/metadata를 더한다. source archive 약31.27 MiB는 공급 증거이며 사용자 runtime 필수 설치량으로 합산하지 않는다. 반복 수집의 임시 실제 비용은 여러 evidence에 남으므로 원 실패 raw는 삭제하지 않았다. 최종 download 시간은 archive/source 각각 result.json에 기록했다. 실제 설치·버전 활성화 latency/RSS·offline JS/cache 비용은 Windows native root 시험 전 미측정이다. [공식 설치 안내](https://docs.deno.com/runtime/getting_started/installation/)의 Windows 지원 안내는 dlpad 최소 OS 실증이 아니다.

현재 ArchivePackage는 단일 source archive의 entry allowlist이므로 ZIP에 없는 LICENSE를 Notice component로 선언하면 실패해야 한다. root 후속 정책 제안: archive extraction과 별도의 정확 revision HTTPS notice source를 explicit typed additional-source manifest로 묶고, 각 URL/size/SHA를 독립 고정하여 original create-only 파일에 bounded 저장한다. final whole-set hash/원 namespace 검증 전 publication·active 전환을 허용하지 않는다. 별도 source의 bytes를 ZIP member인 것처럼 꾸미거나 archive SHA와 notice SHA를 혼합하지 않는다. 혹은 공식 offline `--help`가 license 출력 기능을 안내한다면 root가 실제 고정 exe의 출력·버전·상한·정확 notice를 검증한 뒤 명시 정책으로 채택할 수 있다. 이 조사는 네이티브 실행하지 않았다.

첫 checksum 형식 오인(`windows-deno-supply-first`), 전체 Git tree 4MiB 상한 실패(`...powershell-checksum`), source root directory header 오인(`...bundled-notices`) 원 exit1/raw를 보존했다. 정확 PowerShell Algorithm/Hash/Path filename 비교, 상위/필요 subtree 조회, exact source root directory 허용으로 수집기를 교정했다. 최종 성공이 앞선 실패를 지우지 않는다. 최종 script는 byte/entry/시간 상한·공식 HTTPS redirect allowlist·create-only 출력이며 실제 실행/JS network/계정/사용자 cache 접근이 없다.
