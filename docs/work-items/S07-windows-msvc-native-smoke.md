# S07: 프로젝트 전용 MSVC SDK 실제 링크 시험

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — SDK/std/compiler 공유 경로·통합·native 실행·문서; diagnostic_cache — /tmp collector 초안; metrics_recovery — /tmp 공급 payload 감사; soak_recovery_audit — 독립 검토
- 선행 작업: Clang MSVCtarget COFF 객체 scoped 완료, Rust MSVC std metadata scoped 완료. SDK/CRT 실제 공급은 현재 진행 중이며 완료 확인 후 실행한다.

## 사용자 결과

전역 Visual Studio 설치 없이 프로젝트 도구로 합성 Rust 프로그램을 MSVCtarget에 실제 링크하고 격리 Windows에서 실행할 수 있는지 확인한다.

## 계약과 범위

수정 범위 scripts의 bounded collector 및 docs/validation/evidence/windows-msvc-native-smoke-*와 본 작업. 실제 app workspace/lock/schema 변경은 root 별도조정이며 본 합성시험으로 실제 app acceptance를 대신하지 않는다. 별도 MSVC stdsysroot 및 새 SDK/CRT와 고정 clang/lld를 사용한다. 기존 GNU 도구·산출물·사용자SDK 변경 없음. GNU RUSTFLAGS link-self-contained 옵션을 MSVC에 전파하지 않는다. Rust PathBuf/Instant의 std 연계와 CRT를 사용하는 C header smoke를 시험한다. 필요 라이브러리/옵션 선택은 실제 SDK layout에 따라 root 기록한다.

## 인수 조건

도구·targetstd·SDK 원핀 보존, exactargv nativecompile/link, AMD64 PE 정규 bytes, 합성 프로그램 예상exit/stdout, 원 Job자연0/private 종료/input불변, 실패raw보존. root/독립검토. 실제 dlpad 전체빌드/GUI·cleanOS·서명·release/legal·도구runtime비용은 완료로 표현하지 않는다.

## 검증과 handoff

초안 준비와 SDK 공급 진행 중이다. 실제 native 링크/실행은 아직 없다.

root 초안 통합: C compiler에 원LLVM resource-dir를 명시하여 새alias경로에서 builtinheaders를 추정하지 않는다. Rust는 +crt-static, C는 /MT로 합성fixture의 추가외부VC runtimeDLL 의존을 피한다. 이는 개발 합성test 선택이며 실제 앱stack/정식링크정책으로 확대하지 않는다. 실제 EXE bytes·AMD64 PE헤더·원scriptarchive를 인수한다. 공급splat104초exit0이며 payload독립감사 후 native를 실행한다.

첫 smoke 실제 Rust MSVCcompile와 합성std 실행은 성공했으나 clang-cl의 -fuse-ld=absolutepath driver link에서 program not executable로 실패했다. 원FAIL/headers.obj를 보존한다. C compile을 /c 객체 생성과 지정원lld-link의 별도argv로 분리해 linker discovery에 의존하지 않고 실제 SDK/CRT 링크를 관측한다. 수정후 fresh evidence 재시험한다.

수정1의 Rust compile/run과 C headers compile/명시 link는 실제 PASS했다. C run은 기존 windows-cli-host가 자식argv 최소1을 요구하는데 emptyargv를 줘 host usage로 거절됐다(앱실행전). 원FAIL을 보존하고 합성 --smoke 한 인수를 명시하여 fresh evidence 전체 시험을 다시 실행한다. C fixture는 main(void)이며 이 인수를 사용하지 않는다. 실제 프로그램 stdout/exit0를 확인해야 성공으로 처리한다.


최종 좁은 인수 완료: `python3 scripts/windows_msvc_native_smoke.py --output docs/validation/evidence/windows-msvc-native-smoke-final-20261006-continue --execute-root-smoke`(actualabsolute) exit0. 원 rustcversion/MSVCcompile/std실행, C SDKheaders compile/원lld-link/staticCRT링크/실행 총6native명령이 자연Job0/private종료/input불변으로 통과했다. Rust합성EXE246272B/SHAc6faa491…·C92672B/SHAc6e249d9… AMD64PE, 원15pins/source576map을root·독립 검토했다. [원결과](../validation/evidence/windows-msvc-native-smoke-final-20261006-continue/result.json), [root](../validation/evidence/windows-msvc-native-smoke-final-20261006-continue/root-completion-review.json), [독립](../validation/evidence/windows-msvc-native-smoke-final-20261006-continue/independent-review.json). 원두FAIL보존. Windows ambientenv를 실제unset한근거/cleanOS/SDKallpin/fullMSVCapp/런타임비용/출시완료는 이 좁은 인수에 포함하지 않는다. 다음 전체workspace MSVC실험은 별도작업이다.
