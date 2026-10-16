# S04: 공식 Deno의 관리 factory·독립 helper 실제 시험 staging

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R05, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 새 `/tmp/dlpad-official-deno-managed-helper-native.rs`, 이 문서만
- 선행 작업: 관리 환경 public factory와 원 helper cache handoff; 공식 고정 Deno/EJS offline 증거

## 사용자 결과

기존 공식 Deno bytes/notice 및 bundled EJS preprocessed fixture를 원 앱 관리 package로 배치하여 공개 factory policy Some과 독립 managed guardian을 통해 실행한다. 원 cache 및 node-shim opt-out이 공식 executable의 single-link/원 bytes를 보존하고 실제 bounded offline API를 수행하는지를 후속 native 인수로 연결한다.

## 계약과 범위

AGENTS/README→PRODUCT→CONTEXT→R05/R08/R13→ADR0003 및 INTEGRATIONS, 기존 Windows Deno 공급/offline/node-shim/EJS 기록을 읽었다. 프로젝트 Rust/Cargo는 동결하고 `/tmp` 신규 example만 구현한다. 원 공급을 새로 채택하거나 기존 archive를 재패킹하지 않는다. 실제 계정·영상·site/player JS·network/importremote/UI/shell 실행은 없다.

입력 official executable/원 revision LICENSE/기존 combined solver는 고정 SHA와 크기를 모두 대조한다. package fixture는 실제 원 exe/notice bytes의 full ArchivePackage inventory를 작성하지만 원 upstream ZIP에 notice가 들어있다고 표현하지 않는다. 공급 transitive notice/source 완료는 별도다. public ToolManager factory→begin_execution→원 ProcessSpec Some→managed run_tool로 version/pure API/EJS를 실행한다. helper 자기 등록과 fresh native build는 root 실행 선행 조건이다.

## 인수 조건

Deno exact version/target, 원 copied exe native ID/link1 전후 불변, 원 cache owner bytes 보존, node.exe 부재 및 cache32MiB 사후 논리 예산, bundled preprocessed EJS 정확 output/오류, 원 managed receipts 3개와 finish/reuse, 원 source/license/solver/sentinel SHA 보존. 결과 stderr/stdout는 고정 fixture만 bounded 처리하며 환경 값은 출력하지 않는다. actual ProcessSpec typed 환경/원 receipt 성공을 이전 standalone env collector 성공으로 대신하지 않는다.

## 검증과 handoff

staging 구현 진행. 실제 compile/native 실행은 아직 0이며 root source integration 뒤 수행한다. 기존 evidence의 Deno e020f3…/97,462,048 B, revision LICENSE f62497…/1074 B, EJS combined5d7d7…/159275 B를 다시 대조하도록 runtime fixture에 고정한다. 이전 raw snapshot/도구 SHA는 당시 결과로 유지하고 fresh helper/example/source hashes를 새 실행 evidence에 저장해야 한다.

## 구현 인계

새 `/tmp/dlpad-official-deno-managed-helper-native.rs`는 실제 implementation이다. CLI는 `--fixture-run pinned-deno.exe pinned-LICENSE pinned-solver.mjs fresh-result.json`이다. 입력은 absolute/no-parent/ancestor reparse 거절, no-final-reparse 원 pin과 payload reader의 actual identity 대조 및 exact size/SHA 대조 후 읽는다. 원 세 File을 실행 종료까지 보유하며 최종 원 bytes/hash/native ID를 다시 검사한다.

원 Deno/notice를 새 원 app-managed package namespace에 create-new/stream copy하고 full ArchivePackage + InstallManifest inventory를 만든다. package는 official exe와 원 revision notice의 exact bytes/hash를 포함한다. 원 ZIP 42,630,221 B/SHA `a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238`는 기존 공급 evidence의 metadata를 pin한 것이며 이 example은 ZIP을 재추출하거나 원 ZIP에 LICENSE가 있다고 주장하지 않는다. 실제 공급 채택/installer/notice completeness 인수는 여전히 별도다.

공개 factory와 execution을 사용하여 원 policy Some인 ProcessSpec을 독립 guardian으로 세 번 실행한다. 첫째 official Deno `--version`의 exact 2.9.7/stable/release/Windows target과 3 lines를 확인한다. 둘째 새 로컬 JS의 합계 5000050000/read-net-env denied/원 sentinel read 거절을 확인한다. 셋째 고정 combined official bundle+로컬 preprocessed fixture의 n reverse/sig slice/unknown 및 missing 오류를 기존 증거와 정확 대조한다. 각 실행은 20초, max-log 16 KiB이며 성공/원 terminal confirmation/원 stdout bound를 모두 요구한다. 실제 원 helper control/fixed environment/cache capability 경로가 실패하면 성공으로 바꾸지 않는다.

원 copied exe native identity/link1/97,462,048 B를 세 실행 전후 반복 비교한다. private cache의 bounded postflight 논리 bytes/entry를 검사하며 reparse/hardlink를 거절한다. 원 node_compat_bin/node.exe는 없어야 한다. cache owner raw SHA가 변하지 않아야 하고 actual managed receipts 3개를 magic/slot/stage/256 bytes로 검증한 사본을 남긴다. finish 뒤 원 journal/marker/8 receipts 부재와 fresh begin/finish 재사용을 검사한다. original source/license/solver/copied license/외부 sentinel bytes를 대조한다. cache32MiB는 postflight 조건이고 실행 hard quota가 아니다.

## 실제 검증 상태와 남은 실행

실제 명령 `.local-tools/cargo/bin/rustfmt --edition 2024 /tmp/dlpad-official-deno-managed-helper-native.rs` — exit 0. 새 staged source는 17427 B/SHA256 `9c6c24fe30bb07eaf3986a344f8b37dcdbf915e4611e85ca68f5cfaf2501199b`다. 프로젝트 Rust/Cargo/native 실행은 0이다. 상태는 READY staging 인계이며 시험 인수 완료가 아니다.

후속 root는 이 source를 example으로 게시하고 현재 exact shared factory/config/capture/helper를 fresh compile해야 한다. source 및 compiled example/helper SHA/bytes, current workspace input manifest, 실제 private CLI host Job/input-desktop 불변과 terminal 결과를 새 evidence에 기록한다. 기존 node-shim/EJS evidence snapshot과 fresh 제품 snapshot을 같은 것으로 쓰지 않는다. 입력 official source 및 solver/notice SHA는 이 구현이 실제 다시 확인한다. 실행 root는 소유 temp에서 생성하고 실패시 보존하며 별도 root original ownership proof 없이는 정리하지 않는다. Windows standalone EJS bundle 동등성, upstream player extraction, 실제 사이트, 공급 trust/full notices는 여전히 미검증이다.

## root 통합

원 staging SHA9c6c24fe30bb07eaf3986a344f8b37dcdbf915e4611e85ca68f5cfaf2501199b를 대조하여 crates/engine/examples/windows_official_deno_managed_helper_native.rs에 게시했다. 이전 원 root/변조 수정의 Windows workspace 네 검증은 windows-managed-environment-proof-workspace-20261004T122839Z에서 통과했으며 게시 직전 source/artifact SHA를 root-review에 대조했다. 새 example source 추가 뒤의 native build·공식 실행 및 전체 workspace는 후속 검증이다. root가 새 source/native 실행 담당을 맡으며 shared DB/schema/Cargo.lock 변경 없음.

## 실제 Windows 인수

[최신 빌드·실행](../validation/evidence/windows-official-deno-managed-helper-20261004T123146Z/result.json)은 모두 exit0/private desktop/input desktop 불변/source SHA 불변이다. fixture-evidence.json에 공식 원 Deno2.9.7의 actual managed independent helper3회/원 receipt3개, exact version·read/net/env denied JS·bundled EJS preprocessed 계산과 오류를 기록했다. cache 논리313009 B/12entries, copied Deno97462048 B/native ID·nlink1 불변, node shim 부재, owner 보존·finish/reuse를 확인했다. 전체 elapsed8282ms는 helper/재hash/fixture 비용 포함이며 순수 solver CPU/RSS가 아니다.

root-review.json은 현재 source SHA와 실제 receipt SHA/magic/256bytes, copied executable 원 SHA·pending 부재·node shim 부재를 소유 root에서 다시 대조했다. 실제 upstream player/사이트·Windows yt-dlp bundled 동등성·공급 trust/notice완비/제품 installer·GUI/전체 P0 성공은 아니다. 새 example 포함 Windows workspace 네 검증은 별도 collector에서 진행 중이다.
