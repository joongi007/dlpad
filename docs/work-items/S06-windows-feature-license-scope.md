# S06: Windows feature별 notice 범위 실증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R11
- 담당/수정 범위: root; 본문·새evidence게시/검증, diagnostic_cache /tmp offline metadata/tree만
- 선행 작업: 최신 default+cpu WindowsGNU proof와 Cargo.lock superset notice inventory

## 사용자 결과

다른OS/선택renderer의미포함항목을WindowsCPU필수notice누락으로혼동하지않도록실제feature/target근거를제공한다.

## 계약과 범위

Cargo/Rust/lock변경0, --offline --locked exactWindowsGNU/default+cpu로읽기만수행한다. 최종binary linker reachability나embedded license완비를metadata로증명하지않는다. provisional optional필터결과와공식Cargo tree normal/build결과를구분한다. product자료의coverage_complete=false를유지한다.

## 인수 조건

원Cargo 명령stdout SHA와원lock/currentdefault+cpu feature의일치, 실제 normal/build tree 항목파싱/누락inventory교집합, 다른OS/GPU만있는선택edge를증거로구분한다.

## 검증과 handoff

실제 cargo metadata --offline --locked --format-version1 --filter-platform x86_64-pc-windows-gnu --features cpu exit0, cargo tree --offline --locked --target x86_64-pc-windows-gnu -p dlpad --features cpu --edges normal,build --prefix none --format {p} --no-dedupe exit0. stdlib wrapper는stdout2MiB/시간60초상한을사용하고/tmp만저장했다. root가원metadata/lockSHA·app alternative/cpu/default/software·eframe glow 있고wgpu없음·tree182,897B/259unique와원inventory47중4교집합을독립대조했다. 원metadata raw288/provisional필터270은정확Cargo활성집합으로쓰지않는다. 공식tree보다11개더있었고devfeature처리공백을보존했다. currentworkspaceMITnotice추가후해당3누락은후속후보에서해소하며winapi GNU notice/provenance는미완료다. 다른43항목을삭제하거나전체license완비로판정하지않는다.

원자료: docs/validation/evidence/windows-cpu-feature-license-scope-20261005-resume/root-review.json 및명령·stdout·metadata. [Cargo metadata 원계약](https://doc.rust-lang.org/stable/cargo/commands/cargo-metadata.html), [feature 원계약](https://doc.rust-lang.org/stable/cargo/reference/features.html). 원자료와관측사실을기록하며target/feature변경시다시측정한다.
