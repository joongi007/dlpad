# S07: 관리자 변경 없는 MSVC 개발 도구 공급 검토

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — 경로/공유toolchain/설치실행과검증조정; diagnostic_cache — /tmp 공식자료읽기/기존clang도구metadata비교
- 선행 작업: 지정로컬MSVC SDK/target경로미확인; 기존GNU실제검증유지.

## 사용자 결과

MSVC인수를위해시스템VisualStudio설치를요구하기전프로젝트전용추출도구/SDK공급이가능한지공식자료와실험전계약으로판단한다.

## 계약과 범위

공식Rust/xwin/cargo-xwin/LLVM원문과프로젝트기존clang-cl/lld-link 존재·정확version/packpin 후보를읽는다. downloader실행/라이선스확정/임의패키지설치/UAC/관리자설정은초기읽기범위가아니다. 새로운tooltarget/SDK를실제공급하면root가프로젝트전용경로·정확SHA/크기·runtime비용을기록한다. 기존WindowsSystem/user SDK를삭제/덮어쓰기0; lock/Cargo workspace변경root조정.

## 인수 조건

nativeWindows도구와LinuxcrossSDK추출가능성을분리하며cleanOS/서명/actualMSVCcompile검증없이채택완료표시0. 공식지원/현재버전/sourceURL과한계기록. 시스템관리자변경이필요하면그동작만보류하며다른P0개발계속.

## 검증과 handoff

초기공식읽기전이다. 사용자전용개발의존성다운로드/설치는사용자허용범위이지만현재읽기결과로구체적공급을선택한다.


실제 공급 선행 결과: 고정 xwin0.10.0 Linuxmusl 공식 archive 3,416,863B/SHA d870eb4b…를 새 프로젝트 .local-tools/windows/msvc-supply-20261006에 다운로드했다. 정확 allowlist 4개 regular member만 create-new로 추출했고 ELF xwin7,909,280B/SHA0cb5d482…의 --help를 실제 실행했다. 기존 사용자/시스템 도구 변경 없음. [고정 다운로드 결과](../validation/evidence/windows-msvc-xwin-supply-20261006-continue/download-result.json). SDK/CRT 버전·원manifest/license pin을 이어서 확인하며 SDK 동의/공급/nativeMSVC컴파일은 아직 없다. xwin source가 package manifest SHA검증을 생략하므로 실제 package manifest bytes를 별도 pin해야 한다. 도구 archive비용과 SDK 런타임 비용은 구분한다.


2026-10-06 root 개발 공급 결정: VS16/CRT14.29.16.10/SDK10.0.22621의 프로젝트 전용 개발 도구 공급을 진행한다. 사용자 허용의 무료 개발 의존성/전용 도구 설치 범위다. xwin upstream이 원 광고 manifest checksum을 항상 부정확하다고 명시하는 사실과 실제 원 bytes SHA를 기록하고, 광고 불일치는 보존한다. 개별 VSIX/MSI/CAB payload SHA/size 검증은 유지한다. metadata를 출시 provenance 완료로 표현하지 않으며 공개 재배포·서명·cleanOS/앱링크는 별도다. 실제 VS2019 Build Tools 조건은 이미 고정 원문으로 보관했으며 list/splat의 --accept-license는 개발 도구 설치 동의로 기록한다. 실제 계정·쿠키·비밀 전송이나 관리자 변경은 없다. 먼저 original channel/package 입력을 evidence에 보관한 동일 selector의 list를 bounded 실행하여 실제 다운로드 계획을 확인한다. 새 cache/output만 사용하며 외부 소유 데이터는 건드리지 않는다. 이 결정은 자동 승인 거절을 우회하는 것이 아니다(해당 거절은 없었다).

원 list 명령은 local --manifest와 --manifest-version/--channel을 함께 주어 xwin strict parser exit2로 거절됐다. 실제 network/download/라이선스 flag 처리 전이며 원FAIL 로그를 보존한다. local 고정 channel을 사용하는 경우 충돌 두 옵션을 빼고 새 cache/evidence에서 재시도한다.

고정 local channel의 수정 list exit0, 실제 package bytes11154648/SHA406969c3…는 기존 pin과 일치했다. 원광고SHA불일치와 상위 provenance 미완료는 그대로다. list의67.95MiB/266.89MiB는 SDK CAB 전체를 포함하지 않는 예상치이므로 실제 총 공급비용으로 보고하지 않는다. 동일 root소유 새cache를 사용해 splat을 1800초/로그1MiB/프로젝트 tool cache+출력4GiB 한도로 감독한다. fresh VS16sysroot, x64 desktop, symlink비활성화, 전역 설치/관리자 변경 없음. 실패시 원 산출물을 보존하며 실제 CAB/payload검증과 native링크는 별도 검토한다.


## 실제 개발 공급 인수

project-only xwin0.10.0/VS16 CRT14.29.16.10/SDK10.0.22621 splat exit0·104.136초. [원결과](../validation/evidence/windows-msvc-xwin16-splat-20261006-continue/result.json), [payload 관측](../validation/evidence/windows-msvc-xwin16-splat-20261006-continue/cache-payload-review.json), [출력 inventory](../validation/evidence/windows-msvc-xwin16-splat-20261006-continue/sysroot-inventory.json), [root 검토](../validation/evidence/windows-msvc-xwin16-splat-20261006-continue/root-completion-review.json), [독립 검토](../validation/evidence/windows-msvc-xwin16-splat-20261006-continue/independent-review.json).

SDK/CRT 출력5597regularfiles·607,764,415B/44dirs·symlinkfree, cache1,108,133,055B. 캐시dl38files·311,870,801B에서 payload37개의 SHA는 원실제manifest와 모두 일치했다. CRT3의 광고size가 actualsize와 달라 SHA일치와 size불일치를 분리했고 actualbytes를 pin했다. 원manifest광고SHA 불일치 및 상위publisher/법적재배포 인수 미완료는 보존한다. 기존flatcache 감사의 정규하위폴더 거절은 구조를 실제확인한 bounded재귀열거로 수정했으며 symlink/type/파일변동 거절을 유지했다. tool-개발 공급 기준은 원개별payloadSHA+actualbytes와 전체출력 inventory이며 광고size완비로 표현하지 않는다.

전체파일 inventory는 root가 읽기전후 stat/ID/mtime·streamSHA로 생성했다. 독립담당은 payloadmetadata/SHA 관측·inventory집계·대표SDK/CRT 4file의 actualSHA를 대조했으며 전체5597파일을 다시hash한검증은 아니다. 기존 SDK/사용자자료/관리자설정은 변경하지 않았다. 새SDK가 실제 narrow Rust/C MSVC 링크/실행까지 통과한 근거는 [별도 smoke](S07-windows-msvc-native-smoke.md)다. MSVC실제앱 전체 및 cleanOS/최소OS/서명/정식installer/updater/공개legal·전체P0는 [후속 전체실험](S07-windows-msvc-workspace-experiment.md) 및 별도 게이트다.
