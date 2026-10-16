# S07: Windows MSVC 전체 workspace 실험

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — 공유Cargo/toolchain/통합/actual검증; diagnostic_cache — /tmp flags/cc 원source 설정; soak_recovery_audit — /tmp 독립검토; metrics_recovery — /tmp SDK 공급 검토
- 선행 작업: 원 Clang 도구·MSVC std 및 SDK/CRT 공급, 합성 Rust/C MSVC 실행 scoped PASS. 현재 SDK5597file inventory 생성 완료, 공급 provenance/광고size완비는 별도 미완료.

## 사용자 결과

실제 dlpad 전체 workspace를 새 MSVC target에서 빌드·시험할 수 있는지 확인하고 출시toolchain 선택의 실험 근거를 마련한다.

## 계약과 범위

기존 GNU 산출물과 toolchain/Cargo.lock을 그대로 보존하고 .local-tools/windows/msvc-supply-20261006/target-msvc 새 경로를 사용한다. MSVCtarget만 별도stdsysroot/nativeSDKLib/+crt-static encoded flags를 지정하며 HOSTGNU buildscript/procmacro도구를 유지한다. 원clippy/test/release argv에 exact --target 및 cpu feature/locked/offline를 명시한다. targetCFLAGS는 cached cc1.5.1의 shell-escaped tokenizer로 읽으며 실제shell을 실행하지 않는다. CARGO_BUILD_JOBS=2로 compiler 병렬자원을 제한한다. 변경 범위 scripts collector와 evidence/본 작업. 공유schema/새dependency/lock변경은 root 결정 없이는 수행하지 않는다.

## 인수 조건

source576 map·lock·원GNUCPU/CLI SHA 보존, 원도구pins·actual4명령 각각exit/raw/source/privateJob/input guard. missingcacheddependency/compile/runtimefail을 숨기지 않고 원FAIL을 보존한다. MSVC앱 actualGUI/비용·minimumOS·cleanOS·서명 및 정식installer는 별도다. Windows native ambient 환경 제거는 기존host만으로 입증하지 않으며 명시 targetpaths의 실제실험으로 범위를 제한한다.

## 검증과 handoff

원설정초안만 준비했다. 현재 MSVC 전체 Cargo 명령은 실행 전이다. GNU 검증과 전체P0 미완료 상태를 유지한다.

첫 실험 fmt PASS, clippy는 locked Windows_x86_64_msvc0.52.6 archive가 프로젝트cache에 없어 offline precompile exit101로 실패했다. 원FAIL을 보존하며 새dependency/lock변경 없이 기존 lock의 무료 공개 crate를 Cargo checksum검증으로 공급하는 --allow-download를 추가한다. 원실험이 생성한 target-owner기록과 prior evidence가 일치하는 target만 --reuse-owned-target으로 재사용한다. 새 증거 폴더/effectiveargv를 보존하고 원GNU 실행파일·lock불변을 대조한다.

공급재실험 fmt PASS/locked targetcrate 다운로드완료 후 HOSTGNU quote 빌드스크립트가 gcc_eh/gcc 링크 실패했다. 원FAIL을 보존한다. target의 encodedMSVCflags는 HOST에 적용되지 않기 때문에 original GNU self-contained 옵션을 HOST에만 추가하는 별도 nativewrapper를 준비했고 실제 GNU compile/run·MSVCmetadata무변경/정책8을 검증했다. wrapper7,680B 추가개발도구 및 컴파일마다 CLRstartup미측정비용을 기록하며 Cargo dependency/lock/앱runtime변경은 없다. 원closed Job의 실험target을 동일ownershipmarker로 재사용해 fresh 증거로 전체 재검증한다.

첫helper통합 Cargoquery stdout이비어 실패했다. 원FAIL보존. raw stdout/stderr 전달을고친 새helper9,216B/SHAeaeb1136…를 별도fresh경로에 공급하고 실제-vV hostline/정책8/GNUcompile실행/MSVCmetadata무변경 총7native검증을통과했다. compilerquery명령 및 Cargo통합 proof를 구분하며 새설정으로 다시 전체검증한다.

실제 [stdout 보완 전체 검사](../validation/evidence/windows-msvc-workspace-stdout-20261006-continue/result.json) 네 명령을 통과했다. GUI fixture의 manifest target을 compile 환경에 맞춰 GNU/MSVC로 구분한 뒤 [새 전체 검사](../validation/evidence/windows-msvc-workspace-gui-fixture-20261006-continue/result.json)도 fmt/clippy/test/release·758passed/0failed/13ignored로 통과했다. root는 현재 source576 map·원 raw Job/input guard·test 집계·GNU CPU/CLI/Cargo.lock과 MSVC artifacts를 직접 재해시했고 독립 검토를 보관했다. 원 실패들은 덮어쓰지 않았다.

MSVC CPU SHA121f02cd59b2cd645fd47ae90961b6000ab6547c8883eb64d4e5ef4cd9c76022/8,709,632B, CLI SHAa3001bd29c65523706b0693b25ff2b714515d29d7e7df8feffae3e0e943a2d0b/786,944B다. GNU 대비 각각 +792,064B/+32,256B이며 새로운 dependency/lock 변경은 없다. 독립 PE import 관찰에서 CPU17/CLI5 DLL은 예상 Windows system/API-set 이름이고 동적 UCRT/VCRUNTIME import는 관측되지 않았다. 이는 dynamic load·최소OS·cleanOS 호환성 증명이 아니다. developer compiler wrapper의 호출별 CLRstartup 비용도 미측정이다. 앱 GUI·비용 및 MSVC별 notice graph·최종 스택 채택·정식 installer·서명·전체P0는 별도 인수로 남긴다.
