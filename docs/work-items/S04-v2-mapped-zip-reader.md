# S04: v2 ZIP 원 멤버 mapping의 bounded 추출

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당: metrics_recovery
- 수정 파일: crates/engine/src/tools/bounded_zip.rs, 이 문서만
- 의존성: root 담당 ArchiveLayout::Zip schema/validate 및 installer dispatch

## 범위와 인수 조건

기존 extract_verified_zip API와 v1 flat32entries/256MiB 동작을 유지한다. 명시 v2 Zip layout은 root 아래 안전 ASCII 경로·directory·미선택 regular 멤버를 허용한다. 전체 선언 크기768MiB/4096entries, 선택 component 출력384MiB, 원 archive256MiB를 각각 별도로 제한한다. 선택 mapping의 member→component 이름은 정확히 일치해야 한다. root 밖 경로·case alias·file ancestor 충돌·누락 mapping·암호화·특수 파일·불허 compression·local/central 불일치·중첩 range·DataDescriptor 불일치를 거절한다.

모든 멤버 header/local/data descriptor와 총 선언 크기를 확인한다. 실제 content CRC/SHA와 deflate 소비 바이트 검증은 선택 component만 수행한다. 미선택 content를 decode하거나 CRC 검증했다고 주장하지 않는다. 출력은 caller가 준 한 File에만 쓰며 경로 생성·설치·선택·도구 실행을 하지 않는다. stream 전체를 메모리에 읽지 않으며 부분 출력 정리는 caller 소유다. timeout/cancel은 기존 매 block 검사다.

## 실패 회귀

nested stored/deflate 선택+미선택 멤버 성공, 선택 hash 실패, 누락/case duplicate/root 외부/file ancestor 양방향 실패, 미선택 header 크기/특수파일/local불일치 실패를 추가한다. 기존 v1 이름/CRC/암호화/descriptor/trailing compressed/cancel/deadline 시험을 유지한다. root가 실제 Windows ZIP과 통합 시험을 실행한 뒤 그 범위만 기록한다.

## 실행

Windows collector54275 live 동안 /tmp 초안으로 준비하고 root terminal GO 뒤 게시했다. 개별 rustfmt exit 0. Cargo/native 실행은 담당하지 않았으며 현재 단위·실제 ZIP 성공은 미주장이다.

## 통합 검증과 대소문자 충돌 보강

root는 첫 구현의 ZIP 10개 시험 및 layout 4개 시험 PASS를 실제 실행으로 확인했다. 공식 Windows ZIP은 48항목/전체 선언418,417,814바이트이며 선택 ffmpeg.exe 134,497,280 + ffprobe.exe 134,296,064 + LICENSE 7,651 = 268,800,995바이트다. root 공식 설치 collector53858 exit 0 및 strict engine Clippy88878 exit 0을 확인한 뒤 후속 수정 GO를 받았다.

file ancestor 검사만 대소문자를 구분하던 경계를 ASCII casefold로 보강했다. 선택 member가 실제 존재하는 fixture에서 `supply/FOO` regular와 `supply/foo/bar`를 양 순서로 배치하여 정확히 UnsafePath로 거절하는 회귀를 추가했다. v1 flat 계약은 유지한다. 해당 파일 개별 rustfmt exit 0이며 scoped ZIP 시험은 아래 완료 결과를 기록한다. 이 변화의 fresh Windows 전체/공식 설치 재검증은 root 담당이다.

후속 실제 명령은 프로젝트 전용 PATH/CARGO_HOME/RUSTUP_HOME을 지정한 `cargo test -p dlpad-engine --lib tools::bounded_zip::tests --locked`이다. terminal exit 0, 11 passed/0 failed/0 ignored, 332 filtered였다. `mapped_casefold_file_ancestor_both_orders_rejected`가 실제 양 순서 실패 조건을 통과했다. 최종 담당 source SHA256은 `bf55122910dd0e87b028b6577825e7445df3f65f90d4d493b668219c591b5a16`이며 이후 수정 없이 root 검증에 인계한다.
