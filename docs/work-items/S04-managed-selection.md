# S04: 관리 설치 목록·재선택·active 경로 검증

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: diagnostic_cache agent; crates/engine/src/tools.rs, tools 하위 신규 selection 모듈/전용 tests, docs/work-items/S04-managed-selection.md
- 선행 작업: 관리 소유권/import 및 직접 executable 설치 backend

## 사용자 결과

관리 설치를 제한된 목록으로 확인하고 기존 검증 bytes를 다시 선택해 이전 버전으로 전환한다. 손상 기록/변조 binary/외부 파일은 실행 경로로 반환하지 않는다.

## 계약과 범위

installed(tool)은 최대 128개의 record/metadata 목록이며 binary hash는 선택/active 반환 시 검증한다. entry scan은 최대 2048, 초과는 InventoryLimit로 명확히 실패한다. select_existing(manifest)은 기록 정확 일치·path/ownership·foreign files·256MiB 이내 streaming digest 확인 후 기존 active SQL만 갱신한다. active_installed(tool)은 같은 bytes 검증을 하며 active를 변경하지 않는다. Cargo/schema/UI는 주 담당이 관리한다.

## 인수 조건

정상 기존 version 재선택 가능. 손상 manifest/변조 exe/외부 ownership/foreign files/DB 실패/상한은 이전 active와 files 보존. import 산출물이 --version 실행 증거를 갖는다고 새로 주장하지 않는다.

## 검증과 handoff

`installed(&str)`, `select_existing(&InstallManifest)`, `active_installed(&str)`을 구현했다. 목록은 manifest 최대64KiB/read cap+1, 최대128 설치/2048 filesystem entry(내부 파일·foreign entry 감지분까지 보수적으로 포함)이며 초과 시 InventoryLimit를 반환한다. 목록은 metadata 증거이고 hash/version 확인으로 표현하지 않는다. select_existing/active_installed는 regular/executable mode/path/ownership, exact manifest, foreign files, 최대256MiB의 64KiB streaming hash/동일 Unix file identity를 확인한다. 선택은 active SQL만 바꾸며 기존 설치 파일을 수정하거나 재다운로드하지 않는다. active 조회는 읽기 전용이다.

`cargo test -p dlpad-engine --lib tools::selection::tests --locked`: 5/5 통과, 1.09초. 최종 tools 전체 18/18(1.44초), engine all-targets clippy -Dwarnings 통과. 세 담당 Rust 파일 rustfmt check 통과. 병렬 작업 중 `python3 scripts/check_docs.py` 최종 85 문서 통과.

두 버전 목록·기존 버전 rollback·재시작 후 selected path, binary 변조·손상/초과 manifest·manifest mismatch·foreign file·외부 ownership·SQL trigger failure·잘못된 DB identifier·symlink record·초과 executable·128/2048 상한을 확인했다. 모든 fixture는 tempfile의 합성 자료이며 사용자 외부 설치를 변경하지 않았다. legacy import는 digest 증거만 있고 --version 실행 증거가 없다는 한계를 API rustdoc에 명시했다.

새 schema/Cargo 변경 없음. 설치된 binary가 path 반환 이후 외부 사용자 프로세스로 변경될 수 있으므로 실행 직전 검증/작업 pin은 UI/application 통합에서 확인한다. Windows/macOS 실행·Windows file identity·release UI/통합 검증은 주 담당이 진행하며 미검증으로 남는다.
