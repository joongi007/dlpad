# S10: 중첩 진단 스크롤 접근 복구

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R06, R13
- 담당/수정 범위: root; accessible.rs diagnostic-controls 높이, 본 문서·통합 검증; 전용stress 하네스는별도담당
- 선행 작업: S06 지표 통합, S10-cpu-report-stress

## 사용자 결과

화면 아래에서 실패 보고서를 열어도 복사·형식·저장 제어에 실제 스크롤로 접근한다.

## 계약과 범위

새지표footer가추가된640px 격리화면에서 copytop526/height32로center542가centralclip535밖에멈췄다. nested ScrollArea default min64가외부remainingheight와함께작게예약되는가설을egui0.33.3 실제source로확인한다. root는min_scrolled_height300/max300을설정해outercontent가진단영역을예약하도록한다. API/DB/Cargo/secret처리변경없다.

## 인수 조건

실제clip안의MD/JSON/copy/save/collision 제어 접근, 큰유효report 시험. 숨겨진DoAction/clip확대로회피하지않는다. CPUclippy/tests/release와실제PNG/extents 검증으로판정한다.

## 검증과 handoff

원시full-4~8실패보존/ownedcleanup직접대조. 최소높이실험후새exactbinary로검증진행중이며현재성공을선언하지않는다.

## 인수 결과

최소 높이 예약 후2d6 full-10은 실제 복사·저장·충돌에 접근했으나 HWM511.8MiB로 비용 게이트에 실패했다. 페이지화한1669 [paged-4](../validation/cpu-report-stress-validation.md)는 MD/JSON 전체 복사·저장·충돌·225page 직접선택·원Failed job 보존 PASS, HWM70.8MiB/RSS66.8MiB다. root는 보존된 전체JSON=export, 첫/끝page 및SQLite무결성을 [독립 대조](../validation/evidence/portable-history-policy/root-preserved-artifacts-review.json)했다. 원시 실패들은 보존하며 다른 OS GUI/clipboard/정확2MiB GUI는 별도 게이트다.
