# S04: 고정 Windows Deno offline 실행·비용

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: diagnostic_cache; scripts/windows_deno_offline_native_probe.py, 본 문서, 전용 새 evidence
- 선행 작업: 공식 고정 Windows Deno archive/exe 공급 수집과 original private CLI host

## 사용자 결과

고정 실제 exe의 version/help, fresh 소유 로컬 JS 계산과 권한 미부여 거부를 private desktop CLI Job에서 검증한다. 공급 채택/P0 완료 증거는 아니다.

## 계약과 범위

시작 시 원 source result/ZIP/exe size/SHA를 재대조한다. n.run 원 소유 host/Job과 deadline·input desktop 불변을 재사용한다. help 실제 flag를 확인 후 run --no-config --no-lock --no-prompt와 명시 --deny-read/--deny-net/--deny-env로 실행한다. 공식 eval docs는 암묵적 전체권한이라고 안내하므로 eval 권한검증은 하지 않는다. allow-* 없음, net/env는 query만, read는 이번 소유 synthetic sentinel만. fresh DENO_DIR/TEMP/cache, updater off, import/network/사용자 cache·파일·clipboard·desktop 접근 없음. Rust/Cargo/기존 host 변경 없음.

## 인수 조건

actual version2.9.7/target 일치, native help 지원flags, 계산 정확값, net/env/read query denied, synthetic read NotCapable/PermissionDenied 및 원 sentinel 불변. 원 source/exe/hash불변·host private desktop/Job cleanup·bounded cache. 원 pinned peak RSS 제공이 없으면 null로 기록하고 추측하지 않는다.

## 검증과 handoff

공식 https://docs.deno.com/runtime/reference/cli/eval/ 및 https://docs.deno.com/runtime/reference/cli/run/ primary 조회. 실제 실행 준비 중.

첫 실제 native 4명령의 source/input desktop 불변을 확인했다. version/help/run-help는 exit0, pure 계산은 정확하고 owned sentinel 읽기를 실제 거절했지만 권한 query는 `prompt`여서 기대 `denied` 불일치 exit7이다. `--no-prompt`는 실제 접근을 거절하지만 query state를 denied로 바꾸지 않는다. 원 `windows-deno-offline-native-first` exit1/raw를 유지한다. 실제 도움말 87/91/93행의 지원을 확인한 명시 deny3flags로 다음 fresh 시험을 준비했다. 첫 version은 Deno2.9.7/V8 15.0.245.2-rusty/TypeScript6.0.3/WindowsMSVC target이다. native helper 비용은 각3.4~3.5초 outer 관측이고 Deno 단독 CPU/latency가 아니다.

## 최종 실제 결과와 남은 비용 게이트

`windows-deno-offline-native-explicit-deny`의 4개 native operation 모두 exit0/private-desktop=true/input-desktop-unchanged=true/source_unchanged=true. 계산5000050000, read/net/env query 모두 denied, owned read 거부 true다. 네 operation source_after를 당시 현재 root snapshot과 독립 대조하여 모두 일치했다. 이 대조는 root 후속 포맷 수정 전에 수행했으며 이후 소스에 소급하지 않는다. 두 수집기 세션24484/95718은 terminal이고 소유 핸들이 없다.

하지만 최종 collector는 cache32MiB 비용 상한을 넘어 exit1이다. 원 result/log를 성공으로 바꾸지 않았다. 별도 `cost-audit.json`은 논리 cache97,712,816 B와 최대 node_compat_bin/node.exe97,462,048 B를 기록한다. WSL stat의 원 deno.exe와 cache node.exe가 같은 dev/inode, nlink3 및 같은 SHA로 관측되어 두 시험의 node cache hardlink 생성과 일치한다. 이것은 native Windows FileId를 직접 수집한 결과가 아니며 논리 용량을 물리 복사 용량으로 단정하지 않는다. 원 exe bytes/SHA는 불변이다. 현재 관리 package의 strict nlink1/postverify와 충돌할 수 있어 제품 DENO_DIR/native 실행 후 managed 선택·완료·회수 인수가 필요하다. 기존 사용자 cache는 접근하거나 정리하지 않았다.

실제 elapsed는 original host와 source snapshot 수집을 포함하여 version2.86초/help3.52초/run-help3.45초/pure3.50초다. Deno 순수 실행 latency/peak RSS는 host에서 제공하지 않아 미측정(null)이며 공급 채택·성능 PASS로 쓰지 않는다. --help에서 license를 언급한 행은0개였다. 별도 offline license command가 없다는 완전한 증거로 확대하지 않는다.

최종 수집기는 over-budget 관측을 result에 먼저 기록하고 실패로 유지하며 실패 후에도 원 source/exe postSHA/link count를 기록하도록 보강했다. 관측 hard bound512MiB는 파일량 수집 안전 상한이고 runtime32MiB 허용 budget을 완화한 것이 아니다. 추가 native 재시험은 수행하지 않았고 py_compile exit0만 확인했다. 공급 adoption/P0 미완료, native memory 및 managed hardlink 정책은 root 후속이다.

## 원 source로 분리한 hardlink 원인과 안전한 후속

정확 revision source `cli/node_compat_shim.rs`를 source archive의 bounded stream에서 읽고 별도 node-compat-shim-source.rs.txt/json으로 bytes/SHA를 보존했다. 27–46행은 `DENO_DISABLE_NODE_SHIM` truthy(1/true/yes/on) opt-out, 178–193행은 disabled/active/실제 node PATH 존재 시 no-op, 264–277행 Windows는 원 current_exe를 node_compat_bin/node.exe에 hard_link하고 실패하면 copy하는 구현이다. Run/Task/Test/Bench/Eval/Repl/Serve가 대상이고 49–60행은 cold-start 대상 제한을 명시한다. 실제 관측과 원 source가 일치하지만 이 자료만으로 opt-out native 성공을 주장하지 않는다. [정확 source](https://github.com/denoland/deno/blob/0c071246a412575e07423263404a5d13e7ed6aa2/cli/node_compat_shim.rs), [공식 Node 호환 안내](https://docs.deno.com/runtime/fundamentals/node/), [공식 2.9 안내](https://deno.com/blog/v2.9)도 opt-out을 설명한다.

가장 작은 정합 후속은 fresh copy/원 nlink1 설치 + 앱 소유 별도 DENO_DIR + `DENO_DISABLE_NODE_SHIM=1`을 해당 원 guardian/tool 환경에 명시 전달하고 실제 offline/EJS 및 nlink 전후 회귀로 확인하는 것이다. ProcessSpec/guardian typed 환경 전달을 root가 조정해야 하며 authority를 전역 임시 env나 thread-local로 흘리지 않는다. 현재 source에 no-node shim을 이미 통과한 것으로 표시하지 않는다. 기존 외부 cache/원exe hardlink를 자동 제거하거나 관리 nlink1 정책을 완화하지 않는다. 전용 cache를 다른 볼륨에 둬 copy fallback을 유도하는 대안은 물리 약93MiB를 늘리고 path/volume 정책을 복잡하게 하므로 우선하지 않는다. 기존 실제 Node 설치에 의존한 PATH 우연도 재현 가능한 공급 정책이 아니다. opt-out은 Node child shim 기능을 끄므로 실제 pinned yt-dlp EJS solver에 해당 기능이 필요한지 추가 native 인수가 필요하다.

본 담당 작업은 offline 기능 관측/비용 FAIL 및 읽기 원인 분리 handoff로 종료했다. 공급·제품 관리 설치·cache 정책·P0 전체 완료 상태는 아니다.
