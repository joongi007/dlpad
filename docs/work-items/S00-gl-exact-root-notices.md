# S00: gl-rs 정확 revision root notice

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R13
- 담당/수정 범위: soak_recovery_audit /tmp exact자료·초안; root scripts/windows_candidate_package.py·docs/licenses/upstream/manifest.json 및 새 LICENSE2파일·본 문서·원 검토.
- 선행 작업: 기존15source/28package mapping와 cached VCS/Cargo 원bytes 확인.

## 사용자 결과

개발 후보에 gl_generator/khronos_api 정확 source revision의 원 Apache root LICENSE를 포함한다.

## 계약과 범위

path_in_vcs 부재를 null로 유지하고 exact Cargo.toml.orig SHA로 package·commit·license를 고정한다. 각11358B/총22716B이며 런타임 dependency 증가0. Khronos XML/IDL 추가 MIT형 grant 자료는 별도 미완료이며 root LICENSE로 대체하지 않는다.

## 인수 조건

기존mapping보존, 원grant SHA/size·공식commitURL·원registrymapping일치. License 완비/embedded 완비/SBOM/release_ready 주장을 하지 않는다.

## 검증과 handoff

자료게시와 원mapping 검증 중. 새 native source 전체검증/후보ZIP는 별도다.

root 원2LICENSE·manifest17source/30package·원cached VCS/Cargo mapping PASS. [원검토](../validation/evidence/windows-gl-exact-notices-20261005-resume/root-review.json). 실제 신규ZIP는 후속 측정이며 global complete=false다.
