# S10: Windows 격리 보고서 복사 검증

- 상태: feasibility
- 분류: validation
- 우선순위: P0
- 요구사항: R13, R08
- 담당/수정 범위: diagnostic_cache /tmp 격리probe후보; 주담당실행·통합·본 문서
- 선행 작업: 소유privateWindowStation capability 및현재CPU빌드

## 사용자 결과

Windows 보고서 복사 내용을 검증하되 실제사용자clipboard내용을읽거나변경하지않는다.

## 계약과 범위

LinuxXvfb의이전copy증거를Windows로확대하지않는다. 원래privateDesktop만으로WinSta0clipboard격리가보장되지않는다. ownedWindowStation/privateDesktop에app·observer를명시적으로묶는실험이선행조건이며실패시fallback없음. UI/worker/schema/Cargo변경없음.

## 인수 조건

실제native station명/desktop/PID소유확인,사용자station과분리된clipboard에서UTF8reportheadline/정제sentinel불포함,파일/프로세스/Job정리. capability만통과하면Copy기능통과라고쓰지않는다.

## 검증과 handoff

후보조사중/아직native실행전. 관리자권한·사용자clipboard접근·외부전송없음.

### 격리 clipboard 공급 제약 검토

원 named create-only WindowStation 시험은 Win32 5, unnamed create-only 시험은 Win32 183으로 실패했다. `/tmp/dlpad-windows-clipboard-station-feasibility/REVIEW.md`를 주 담당이 검토했다. private desktop만으로 clipboard는 격리되지 않으므로 실제 사용자 clipboard 시험으로 대체하지 않는다. 관리자 권한 변경·기존 station 채택·DACL 수정·전역 입력은 수행하지 않았다. 현재 Windows Copy report의 격리 종단 검증은 미완료이며 Linux 증거로 완료 판정하지 않는다. 진단 P0 기능의 다른 독립 검증은 계속 진행한다.
