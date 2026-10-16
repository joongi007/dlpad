# S07: 최신 Windows 개발용 패키지 갱신

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root scripts/windows_candidate_package.py의 최신 실행·새 전용 evidence·문서; 하위 원ZIP 읽기 검토만
- 선행 작업: 최신540소스 defaultCPU전체 검증, 정확Winapi grant binding 및 GUI60초 측정 종료

## 사용자 결과

현재 구현과 검증한 정확 바이너리를 재현 가능한 개발용 ZIP에 연결한다.

## 계약과 범위

최신source/CPU SHA/전체4명령 proof를 묶어 동일입력2ZIP SHA를 비교한다. notice mapping28원source와 workspace MIT·exactWinapi payload를 검증한다. 프로젝트 자체원LICENSE를 그대로 넣는다. 공개배포·서명·설치완료·원생성SDK/embeddedlegal 검수완료로 표시하지 않는다. 기존Git을초기화하지 않으며 usermedia/credentials를묶지 않는다.

## 인수 조건

동일바이너리·원소스·Cargo.lock·notice원SHA 보존, 2ZIP bit-identical, manifest release_ready=false와미검증범위 명시, 각entry size/hash·actuallicense inventory 및검증source proof를 root와 독립담당이 대조한다.

## 검증과 handoff

기존notice44누락 후보는이전CPU snapshot이다. 최신후보생성은현재GUI측정이끝난뒤직렬로진행한다. 공개릴리스/실제CI는수행하지않는다.

원CPU3f9d8825f1674b1c682094506f8774df208494dc10ccf1e0655f62d06b8e3667(7888384 bytes)/최신전체4명령 proof로 두ZIP을 생성했다. windows-current-candidates-20261005-continue 원ZIP은 각각12682630 bytes/863entries, SHA83ca7bc19dae33357097d2df5c52f24d6592d7e068aa9441adb148d22b8ccf9a로 동일하다. root가 모든entry content/hash/CRC/metadata·원binary·Cargo.lock/toolchain/proof·build evidence 원문을 대조했다(root-review.json). 실제 wholelock 누락43개이며 complete/release_ready/embeddedlegal/independentrebuild는false 유지. 독립 rawZIP 검토가 진행 중이다.

독립담당이 actual2ZIP 전체863항목과각license原SHA·Winapi 두grant의원rootGitblob·현재540source/checks/proof/pins를확인했다. root가원JSONSHA를대조하고 independent-review.json에통합했다. 본개발후보갱신 범위done이며 공개릴리스·서명·installer·embeddedlegal·remoteCI는완료가아니다.
