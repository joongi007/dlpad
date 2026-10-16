# S07: 개발 CLI의 명시 설치 선택·재검사

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — 기존 binary 통합·상태/문서·검증; metrics_recovery — /tmp dlpad-local-install.rs 두명령 초안; diagnostic_cache — /tmp 옵션/실패경로 읽기검토; soak_recovery_audit — 현재 engine 실제 인수 후 원CLI 명령 증거 검토
- 선행 작업: engine ExplicitActiveSelection native 인수 후 제품 파일에 CLI초안을 적용한다. 현재 sourcefreeze에서 기존 binary를 수정하지 않는다.

## 사용자 결과

사용자는 개발 CLI에서 이미등록된묶음+receipt를 새active 선택 root에 명시 선택하고, 다음실행에서 동일세경로를readonly검사한다. 이전7명령과사용자데이터보존을유지한다.

## 계약과 범위

select-active / inspect-active에 --target 등록root --receipt-target 외부receipt --active-target 선택root와 선택timeout을고정 argv로받는다. parser최대11/UTF16경로상한/strictflags가새명령에도적용된다. stdout<=4KiB·phase/mutation/pre-dropcaps/afterexitcaps없음·historicalauthority 미주장를분리하고rawpath/SID/전체records를내보내지않는다. 자동discovery/launch/upgrade/삭제·P0전체완료를추정하지않는다. manifest/sourceprepare와외부도구registry권한을공유하지않는다.

## 인수 조건

실제현재pair선택→별도 CLI readonly검사, 다른유효pair혼합/unknownnamespace/이미있는target/native case-verbatimalias overlap/취소·기한·strictparser실패를보존한다. engine actual proof, Windows 전체4검사, CLI원외부bytes/IDs/root+독립검토를대조한다. sourcefreeze동안읽기초안만진행하며전체검증전완료표시없음.

## 검증과 handoff

현재 engine whole실행중·CLI초안 단계다. 새deps/lock변경없고실제크기는release뒤기록한다. 동시같은파일수정0·root공유계약조정.

root engine actual6cases/root+독립인수 완료후 CLI 초안을 통합했다. 같은파일동시수정없음. select후verify실패는mutationtrue/rootcaptrue/filecap1로보존하고 inspect는readonly3명시경로만사용한다. 현재새두명령은실제컴파일·인수전이며Windows전체4검사를시작한다.

Windows전체4검사·753passed/13ignored 및 실제CLI14case는root/독립검토PASS다. CPU53d12740…/7,917,568B불변, CLI0b30c162…/723,456B(+71,680B), deps/lock불변. [whole](../validation/evidence/windows-active-cli-workspace-20261006-continue/root-review.json), [actual14](../validation/evidence/windows-active-cli-real-20261006-continue/root-review.json). 실제CLI취소/기한은빠른8MiBpair에서handlerready관측이없어시간추측으로시험하지않았고미검증유지. default-OFF 원event readiness fixture 후속으로추가해범위인수를끝낸다. 이시점전체workitem은in-progress다.


2026-10-06 최종 범위 인수: [현재 기본 CLI14 주 담당 검토](../validation/evidence/windows-active-cli-current-runtime-real-20261006-continue/root-completion-review.json)와 [독립 검토](../validation/evidence/windows-active-cli-current-runtime-real-20261006-continue/independent-review.json), [기본 전체4검사](../validation/evidence/windows-selected-process-final-workspace-20261006-continue/root-completion-review.json)를 대조했다. 현재 source575는 항목 수이며 CPU bb89e182…/7,917,568B, 기본 CLI5dc933c…/723,456B, 전체755passed/13ignored·설치89회귀다. 원control의 별도 featureCLI b55243bb…/724,480B는 [실제 준비신호 취소·기한](../validation/evidence/windows-active-cli-control-real-20261006-continue/root-review.json) 및 같은 디렉터리 independent-review.json의 좁은 select-active prepair 인수다. 기본 CLI14에 signal 인수를 소급하지 않는다. 현재14의 새 nativeID 조회는 없고 원 engine ID 증거와 원 파일 SHA 보존을 구분한다. deps/lock 불변·자동 실행/installer/upgrade/전체P0 완료는 아니다.
