# S00: Windows 한글 폰트 소유 버퍼의 bounded allocation

- 상태: done (Windows 합성 empty-library steady-idle 범위)
- 분류: fix
- 우선순위: P0
- 요구사항: R06, R11, R13
- 담당/수정 범위: metrics_recovery — `/tmp/dlpad-accessible-font-allocation.rs`, 이 문서만
- 선행 작업: 원 Windows steady-idle 첫 회 WS 80MiB 초과, root GUI/source freeze

## 사용자 결과

한글 폰트 전체 바이트를 metadata 길이에 맞춰 확보해 읽는 초안이다. 실제 Windows WS 감소와 한글 렌더 보존은 다음 fresh 비용/PNG 검증에서 확인한다. WorkingSet trim 또는 budget 인상은 하지 않는다.

## 계약과 범위

원 `accessible.rs`를 복사해 font_bytes 내부 읽기만 helper로 분리한다. metadata 원 길이 0 또는32MiB 초과는 읽기 전에 거절한다. fallible try_reserve_exact 후 정확한 길이만 resize/read_exact하고 마지막1byte로 성장 여부를 검사한다. 원 크기보다 짧아졌거나 커졌거나 I/O/할당 실패하면 None으로 기존 다음 후보 fallback을 유지한다. arbitrary stream 전체 적재와 파일 dump는 없다. context/font family와 UI의 다른 동작은 유지한다.

원 malgun.ttf 크기는13,459,196B다. 기존 빈Vec read_to_end의 retained capacity는 아직 실측하지 않았다. 새 방식은 원 길이에 맞춘 소유 allocation 요청이며 실제 allocator capacity가 정확히 동일하다는 보장은 없다. resize의 zero 초기화와 EOF검사1회 비용이 추가된다. WS 개선량은 아직 주장하지 않는다. root의 CloseWait staging과 같은 accessible.rs라 전체파일 덮어쓰기 대신 해당 함수/helper/테스트 diff만 통합해야 한다.

## 인수 조건

합성 short chunk reader의 전체 내용 보존, 원 길이보다 성장/축소한 reader 거절, oversized/empty 선언 시 reader 호출0 시험을 포함한다. root가 통합 후 실제 CPU 앱 단위시험과 fresh3회 전체 Job idle 비용 및 한글 ready/end PNG를 확인한다. 80MiB/1% 목표를 유지하며 실패 원시 결과를 남긴다.

## 검증과 handoff

실행: `.local-tools/cargo/bin/rustfmt --edition 2024 --config skip_children=true /tmp/dlpad-accessible-font-allocation.rs` exit0.

원 `crates/app/src/accessible.rs` SHA256 `0d36029800be74bbed6c3f805048d04e042bbd7b24cc60acba6241205a3aaed8`; staged SHA256 `4f90a583cb61bd2f57725b2599afdba83544b4da5790e4affb3f27e61296b594`.

프로젝트 Rust/Cargo 변경0. 단위시험 실행/compile/native GUI 비용/실제 한글 PNG는 아직 미실행이다. root가 source freeze 종료 후 최소 diff 통합과 검증을 담당한다.

## 최종 통합 및 scoped 인수

root가 staging SHA를 대조하여 최소 font 읽기 변경을 통합했다. `windows-closewait-feature-regression-20261004T133915Z` 실제 Windows feature workspace에서 bounded_font_tests의 short chunks/성장·축소/oversize·empty 시험이 CPU 및 accessible 대상 모두 통과했다. `windows-font-closewait-default-workspace-20261004T134018Z`의 default fmt/clippy/test/release 네 명령도 통과했고 root가 원 snapshot SHA를 독립 대조했다.

[font 개선 후 fresh3회 결과](../validation/evidence/windows-idle-font-exact-20261004T134258Z/gui/result.json)는 root의 원 source/binary/collector identity 확인과 한글 PNG glyph 검토를 포함한다. CPU는 0.31135286/0.31060837/0.30969038%, sampled WS는82,173,952/76,140,544/76,206,080B, private는46,870,528/46,735,360/46,739,456B다. 세 회 모두80MiB/1%를 충족하고 자연 exit0/Job0/desktop close/input 불변을 확인했다. 이 수직 인수 범위에서 done으로 마감한다.

기존 첫81.3MiB WS 초과 및 전체 FAIL은 그대로 보존한다. 기존 font capacity를 실측하지 않았으므로 절감 전체의 인과를 allocator capacity 하나로 확정하지 않는다. 새 수치는 합성 empty-library·준비 후10초·fresh3회 관측이며 quiet_host_verified=false, 외부 호스트 부하 분해 미검증이다. 실제 IME/reader/장시간GUI/영상 실행/최종 스택 채택 완료로 확대하지 않는다. 현재 source freeze를 유지하며 근거 없는 추가 메모리 변경은 하지 않는다.
