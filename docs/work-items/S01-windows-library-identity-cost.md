# S01: Windows 목록 identity 비용 분리

- 상태: done
- 분류: performance
- 우선순위: P0
- 요구사항: R01, R03, R08
- 담당/수정 범위: root 새 crates/engine/examples/windows_library_identity_cost.rs·전용 증거·본 문서, diagnostic_cache /tmp 초안만
- 선행 작업: S01-library-prepared-reconcile-cost 실제 native 비교와 기존 취소/identity 회귀

## 사용자 결과

목록의 주기적 CPU 초과를 해결하기 위해 파일 식별 호출의 비용을 실측한다. 실험 구현을 제품 성능 해결로 표시하지 않는다.

## 계약과 범위

기존 public file_identity API의 full chain을 소유 임시 synthetic10000파일에서 동일순서로 실행한다. 원file HANDLE finalpath/path canonical equality, metadata len/mtime, opaque128bitID를 유지한다. bounded phase wall counters와 원process CPU를 구분한다. streamed entry, 고정3round, 고정flag, 한정 JSON 출력만 사용하며 제품함수/정책·schema·Cargo·deps는 변경하지 않는다. Windows 첫 릴리스 범위이며 다른OS측정은 하지 않는다. root의 현재 전체검증 source freeze가 끝난 후에만 게시한다.

## 인수 조건

Windows native 실제compile/run, 매회10000 native identity와 count/creation/ticks 확인, originalsource SHA·naturalJob0·private/input desktop 불변, own sentinel와 fixtures 보존. 각 phase wall은 CPU가 아니며 원public reconcile/GUI 전체비용을 대신하지 않는다. 결과로 안전 계약을 유지할 다음 구현을 판단한다.

## 검증과 handoff

현재 /tmp 초안 준비 중. perfile SQL prepare 재사용은 CPU평균약1.77% 변화로 예산 해결을 입증하지 못했다. 다음은 open/finalpath/canonicalize/metadata/nativeID/format의 실제 비용이며, 대기 예산과 직접 scan 인과는 별도 검증한다.

하위 초안은 원CPU creation·kernel/user단조 검증과 owned fixture 영구 보존을 보강했다. 원production에 없던 추가 zeroID거절은 제거했다. root가 실제GUI sourcefreeze가 종료된 후 게시/compile/run하며 현재 초안만 존재한다. 원초안 독립검토의 지적과 변경본 SHA dfad3162…를 보존한다.

root가 수정본 fullchain과 독립 검토를 확인하고 새 example을 게시했다. 실제Windows release compile PASS, 소유Job 원fixture 비용실험 실행 중이다. 생산 scanner 변경0, 전체all-target는 후속.

실제 Windows fullchain3회 PASS, 전체CPU1.0625/1.0625/1.046875초. 각단계 wall 중 canonicalize약0.356~0.367초, path metadata0.266~0.271초, openregular0.207~0.212초가 컸다. phase wall을CPU로단정하지않고 생산scanner/GUI 인과를구분한다. 매회10000/opaqueID/원creation/checkedCPU/phasecount·sentinel과원source/Job0/private/input을root검토했다. 새537소스 전체fmt/clippy/test/release 진행 중이다. 기존정책을 유지할 metadata 호출 최적화의 후속 검토가 필요하다.

최신537소스 fmt/clippy/test/release 실제 PASS와 root SHA/Job 종료 검토를 완료했다. 본 실험 수직 작업만 완료이며 생산성능/GUI 예산은 미완료다. 후속 metadata 최적화는 안전성 검토·별도작업계약·실측·동일GUI회귀가 필요하다.
