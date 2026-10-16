# S04: bounded ZIP archive 공유 계약

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: root — tools.rs/archive_manifest.rs/Cargo.toml/Cargo.lock/공유 문서; diagnostic_cache — bounded_zip.rs/전용 작업 문서
- 선행 작업: 관리 runtime 실제 코드 감사, Linux 공급 실험

## 사용자 결과

관리 archive의 크기·전체 SHA와 각 component의 크기·SHA를 별도로 검증하며 상한 내 스트림 추출을 제공한다. 이후 설치 worker·명시 적용·제거를 연결할 기반이며 이 단위만으로 관리 설치 완료를 주장하지 않는다.

## 계약과 범위

rawzip0.5.1의 caller buffer 기반 directory iterator와 flate2 1.1.10 rust_backend를 사용한다. 둘은 root가 exact Cargo pin을 조정한다. ZIP32/ZIP64 실제 파싱은 라이브러리에 맡기며 전체 archive/stream을 RAM에 적재하지 않는다. deflate/stored만 허용하고 dictionary가 큰 다른 codec는 제외한다. 라이브러리 설명은 https://docs.rs/rawzip/0.5.1/rawzip/ 와 https://docs.rs/flate2/1.1.10/flate2/ 의 원문을 기준으로 한다. 최종 크기/비용은 실제 build 근거로 기록한다.

ArchivePackage는 version1·Managed ownership·tool/version/target/source_url/license·archive_bytes/archive_sha256·최대8개 flat component(name,bytes,sha256,role)를 갖는다. archive 및 expanded 각각256MiB, directory count32, 이름ASCIIidentifier만 허용한다. exact requested name component만 추출하며 전체 directory를 먼저 검증한다. 실행권한/게시/DB/삭제/GUI 연결은 후속이며 extractor는 호출자가 제공한 File capability에만 쓴다.

## 인수 조건

digest/CRC/크기/메서드/암호화/duplicate/path/type/entrycount 실패를 reject하고 cancel/deadline 확인, 원 source/foreign 보존. Rust unit fixture와 실제 Linux 공식 ZIP 자료를 각 범위로 검증한다. 새로운 공급 catalog 활성화와 license/trust 채택은 별도다.

## 검증과 handoff

현재 공유 schema와 독립 parser 구현 중. 기존 Windows/Linux 전체PASS는 이 변경 이전 snapshot이다.

## root 실제 Linux 공식 ZIP 인수

의존성 fetch exit0, Cargo.lock rawzip0.5.1 하나 추가; rawzip crate256630bytes/MIT/MSRV1.85를 원 registry manifest에서 확인했고 exactRust1.94 계약을 유지했다. flate2 1.1.10/miniz backend는 기존 lock packages를 재사용한다. 설치 공급 archive·notice/source trust는 이 dependency 확인과 별도다.

`cargo build -p dlpad-engine --example verify_managed_archive --release --locked` exit0. source를 동결한 release example569144bytes/SHA3d84aa3da244ad12642ddb848d9a0351ba21a818714542f02d72503407eec704를 실제 기존 공식 ZIP에 argv로 실행했다. native exit0/PASS, archive41596794bytes/sourceSHA전후동일c6527f24..., output95830104bytes/SHAce6a052b..., elapsed1562ms/Linux VmHWM2388KiB. output은 anonymous tempfile이며 설치·실행·chmod·active 전환은 하지 않았다. peak은 이 parser example 한 프로세스의 표본이며 전체 GUI나 설치 worker 비용이 아니다. 증거 docs/validation/evidence/managed-archive-native-fixture-first/{package.json,native-stdout.json,root-run.json}.

독립 review에서 rawzip CRC verifying_reader가 descriptor를 자동 검증하지 않는 점을 발견해 원 descriptor를 central 값과 직접 대조하고 변조 회귀를 추가했다. trailing deflate payload 소비 및 header/data 영역겹침도 거절한다. 하위 scoped7testsPASS 보고 후 root전체workspace runner를 실제 실행했으며 terminal 전 전체통과로 표시하지 않는다. 이전 path 공격 fixture는 ZIP writer가 정상화해 실제 공격이 아니었던 첫1FAIL을 보존하고 raw header 변조로 수정했다.

## 최신 전체 검증 실행 중

`python3 scripts/rust_workspace_validation.py <canonical-project>/docs/validation/evidence/managed-archive-linux-workspace-first --execute` 실제 실행. 기본 fmt/clippy/test를 지나 release를 실행 중이며 CPU 명령까지 terminal을 확인한다. Windows는 native fresh `cargo build -p dlpad-engine --bin dlpad-process-guard --locked`가 exit0/sourceunchanged/inputdesktopunchanged를 통과했고 이어 `windows_current_workspace_probe.py .../managed-archive-native-workspace-first --capability --execute`를 실행 중이다. 두 source snapshot collector가 끝날 때까지 프로젝트 Rust/Cargo/vendor/runner를 동결한다. 후속 installer 초안은 별도 /tmp 파일과 신규 workitem에만 작성한다.

## root Windows archive 변경 전체 terminal 검토

managed-archive-native-workspace-first terminal exit0, capability/fmt/CPU clippy/test/release 각 exit0/pass/inputdesktopunchangedtrue. root가363개 source_final SHA를 현재파일과 대조해 불일치0. test.log에서 ArchivePackage3개 및 bounded_zip7개 실제 native 시험이 모두 ok임을 별도로 확인했다. CPU exe7452160bytes/SHA4a9e0dd5...는 새 구현 포함 빌드지만 GUI에 archive 설치를 연결하지 않아 LTO로 미사용 경로가 제외될 수 있다. 이 크기를 최종 설치 UI/압축기 전체비용으로 주장하지 않는다. 실제 공식 ZIP95.8MB 추출 인수는 Linux example 근거이며 Windows/macOS의 실제 공급 ZIP 인수는 별도다. Linux CPU 전체는 아직 실행 중이며 소스 동결을 유지한다.

## root Linux 최종 전체 terminal

managed-archive-linux-workspace-first collector terminal exit0/workspace_checks_complete=true. 기본 fmt/clippy/test/release와 CPU clippy/test/release 전부 exit0/pass. root가 source_final을 현재 파일과 SHA 대조해 불일치0을 확인했다. native Windows363파일대조/전체4명령 및 실제 Linux ZIP 인수와 함께 이 bounded archive 계약·추출 API 범위를 완료한다. source freeze는 두 collector의 terminal/SHA 검토 후 해제한다. 전체 관리 설치 수직 기능과 P0/최종 renderer/3OS/릴리스가 완료된 것은 아니다. 다음은 새 stage nativeownership/publication/checkpoint 및 package-aware selection/removal을 설치 worker/UI까지 연결한다. /tmp draft는 컴파일되지 않은 초안이며 제품 구현으로 계산하지 않는다.
