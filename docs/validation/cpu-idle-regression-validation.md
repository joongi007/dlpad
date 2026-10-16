# CPU idle 비용 회귀 진단

2026-10-03. diagnose 스킬로 기존 실제 성능 loop를 read-only 재사용했다. 제품/source/probe 변경은 주 담당이 맡고 본인은 새 evidence/전용 문서만 작성한다.

## 고정 후보 재측정

Binary SHA256 `522e2304fc38618df696cfca1f126e08bc77634f48ac1db631d3c797fe8f8cbd`,12,327,288bytes. 기존 root `accessible-performance/cpu-parity/result.json`은10.013521초/idle1.098515%/RSS53024KiB로 proposed idle<1% 초과였다. 같은 byte hash로 다음 명령을 한 번 수행했다.

```text
python3 scripts/accessible_performance_probe.py target/release/dlpad-cpu docs/validation/cpu-idle-regression/baseline-repeat --font /mnt/c/Windows/Fonts/malgun.ttf
```

exit0.10.012881초 idle0.499357%,RSS peak sample52796KiB. owned Xvfb/privatebus/temp freshdata,2초warmup+10개1초sleep sampling의동일loop다. idle CPU는app process ticks/wall seconds이며 machine 전체normalized CPU가아니다. screenshot은sampling뒤라sampling구간의CPU에포함되지않는다. 이전 GUI/Cargo/Windows CLI 종료 조건을root가확인한뒤 실행했다. processID/startticks와memorymap/rawPNG는baseline-repeat에보존했다.

같은binary 최초기준초과가이번표본에서는재현되지않았다. 두표본중하나는초과하므로안정적으로<1%라고확정하지않는다.10초표본에서단몇clockticks 차이는percentage를쉽게바꿀수있다. 원래1.0985% 결과를삭제하거나성공으로쓰지않는다. source는본인변경없음/rootfreeze계약, 실행binary는probe시작/끝hash불변확인. 별도baseline source-before hash는수집하지않았으므로실측동일source proof를과장하지않는다.

## ranked 가설과 다음 대조

1.250ms activation UI repaint가CPU를소모한다면root의callback-driven restore wake로poll repaint를제거한새후보에서동일loop 비용이감소한다.
2.IPC worker50ms polling이주요비용이라면UI repaint만제거한후에도worker wake/ticks가유지되며IPC wait를event-driven으로바꿔야더감소한다.
3.호스트부하/clocktick sampling노이즈가주요원인이라면같은binary 재실행만으로<1%가되며패치후단일낮은값만으로patch인과를확정할수없다. 이번재측정은이가능성을지지하나확정하지않는다.

root가activation callback 변경을조정한다. 새READY후동일loop1회만수행해피드백을얻는다. source읽기만으로특정worker를범인으로확정하지않는다. 다른OS/숨김복귀정확성은root별도native시험이며idle수치로대체하지않는다.

## 정리 상태

소유 GUI는probe 종료 시SIGTERM(-15)으로회수됐다(사용자정상close 경로검증아님). owned GUI/bus reaped=true,temporary_fixture_removed=true,Xvfbcontext종료. 새GUI대기,baseline evidence보존. 제품회귀수정/정확한StopExit는이성능collector의인수조건이아니다.


## callback-only 동일 loop 회귀

주 담당은 runtime 변경을 activation wake hint와 unconditional250ms UI repaint 제거로 제한한 후보를 제공했다. Metrics/large fixture 통합은 포함하지 않았다. root는 Linux IPC coalesce/late-bind/callback-panic retention1test 및 CPUalltargetclippy/release PASS를 보고했다. callback이 숨김 복귀를 깨뜨리지 않았다는 최종GUI 판정은root별도시험이며 본성능loop로대체하지않는다.

```text
python3 scripts/accessible_performance_probe.py target/release/dlpad-cpu docs/validation/cpu-idle-regression/callback-only --font /mnt/c/Windows/Fonts/malgun.ttf
```

Actual exit0, binary SHA256 `b8bbf93a2cc4b05dfe202edfe1c92fca6aff7566b694eb63f4427df7d7a87409`,12,329,336bytes.10.012622초 idle0.099874%,RSS sample max52800KiB. 직접 재측정baseline0.499357%보다약0.3995percentage-point 감소했고메모리는거의같다. 제안<1% 기준을완화하지않았다. 이단일비교는250ms repaint 가설을지지하지만최초1.0985%와측정노이즈를완전히분리하거나모든idle조건을증명하지않는다. IPC50ms workerpoll이완전히없어진것으로주장하지않는다.

시작/끝binaryhash불변확인,owned GUI/bus reaped=true,temporaryfixture removed=true,Xvfb종료. 제품/probe수정없음. 기존baseline/최초root실패원시는모두유지한다. comparison.json은두실제표본을요약한다. callback-source-final.json은관측뒤source/probehash이며미수집beforehash를대체하지않는다. throwaway source/log instrumentation을추가하지않아삭제할디버그코드는없다.

회귀수집작업은완료했으며root가source통합·복귀nativeGUI·최종product 검증을책임진다. 추후idle회귀예방은unconditional repaint 대신실제worker/IPC wake를UI에전달하는event seam을유지하는것이다. 정확한idle비용은realUIloop로측정해야하며callback단위시험만으로CPUbudget을대체하지않는다.
